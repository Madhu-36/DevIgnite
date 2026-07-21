use super::{SmokeTestConfig, SmokeTestOutcome, SmokeTestSuiteResult, TestCase};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub struct SmokeTestRunner {
    timeout: Duration,
}

impl SmokeTestRunner {
    pub fn new() -> Self {
        Self { timeout: Duration::from_secs(30) }
    }

    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self { timeout: Duration::from_secs(timeout_secs) }
    }

    pub fn run_suite(&self, config: &SmokeTestConfig) -> Result<SmokeTestSuiteResult> {
        let start = Instant::now();
        let mut outcomes = Vec::new();
        let mut all_passed = true;

        for tc in &config.test_cases {
            let outcome = self.run_single(tc)?;
            if !outcome.passed { all_passed = false; }
            outcomes.push(outcome);
        }

        Ok(SmokeTestSuiteResult {
            language: config.language.clone(),
            version: config.version.clone(),
            all_passed,
            outcomes,
            total_duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    fn run_single(&self, tc: &TestCase) -> Result<SmokeTestOutcome> {
        let start = Instant::now();
        let test_timeout = Duration::from_secs(tc.timeout_secs).min(self.timeout);
        let binary = Path::new(&tc.command);

        if !binary.exists() {
            return Ok(SmokeTestOutcome {
                test_name: tc.name.clone(),
                passed: false, exit_code: -1,
                stdout: String::new(), stderr: String::new(),
                duration_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("Binary not found: {}", tc.command)),
            });
        }

        let mut cmd = Command::new(&tc.command);
        cmd.args(&tc.args).stdout(Stdio::piped()).stderr(Stdio::piped());

        if let Some(ref stdin_data) = tc.stdin_input {
            cmd.stdin(Stdio::piped());
            let mut child = cmd.spawn().context("Failed to spawn")?;
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                stdin.write_all(stdin_data.as_bytes())?;
                drop(stdin);
            }
            let output = child.wait_with_output()?;
            return self.build_outcome(tc, output, start);
        }

        let output = cmd.output().context("Failed to execute")?;
        self.build_outcome(tc, output, start)
    }

    fn build_outcome(
        &self,
        tc: &TestCase,
        output: std::process::Output,
        start: Instant,
    ) -> Result<SmokeTestOutcome> {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        let pattern_ok = tc.expected_stdout_pattern.as_deref()
            .map(|p| stdout.contains(p))
            .unwrap_or(true);

        Ok(SmokeTestOutcome {
            test_name: tc.name.clone(),
            passed: exit_code == tc.expected_exit_code && pattern_ok,
            exit_code, stdout, stderr,
            duration_ms: start.elapsed().as_millis() as u64,
            error: None,
        })
    }

    pub fn quick_verify(binary_path: &str) -> Result<bool> {
        if !Path::new(binary_path).exists() { return Ok(false); }
        let output = Command::new(binary_path)
            .arg("--version")
            .stdout(Stdio::piped()).stderr(Stdio::piped())
            .output()?;
        Ok(output.status.success())
    }
}
