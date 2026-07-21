use super::{SmokeTestConfig, SmokeTestOutcome, SmokeTestSuiteResult};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub struct SmokeTestRunner {
    timeout: Duration,
}

impl SmokeTestRunner {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn run_suite(&self, config: &SmokeTestConfig) -> Result<SmokeTestSuiteResult> {
        let suite_start = Instant::now();
        let mut outcomes = Vec::new();
        let mut all_passed = true;

        for test_case in &config.test_cases {
            let outcome = self.run_single_test(test_case)?;
            if !outcome.passed {
                all_passed = false;
            }
            outcomes.push(outcome);
        }

        let total_duration_ms = suite_start.elapsed().as_millis() as u64;

        Ok(SmokeTestSuiteResult {
            language: config.language.clone(),
            version: config.version.clone(),
            all_passed,
            outcomes,
            total_duration_ms,
        })
    }

    fn run_single_test(
        &self,
        test_case: &super::TestCase,
    ) -> Result<SmokeTestOutcome> {
        let start = Instant::now();
        let test_timeout =
            Duration::from_secs(test_case.timeout_secs).min(self.timeout);

        let binary_path = Path::new(&test_case.command);
        if !binary_path.exists() {
            return Ok(SmokeTestOutcome {
                test_name: test_case.name.clone(),
                passed: false,
                exit_code: -1,
                stdout: String::new(),
                stderr: String::new(),
                duration_ms: start.elapsed().as_millis() as u64,
                error: Some(format!(
                    "Binary not found at: {}",
                    test_case.command
                )),
            });
        }

        let mut cmd = Command::new(&test_case.command);
        cmd.args(&test_case.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(ref stdin_data) = test_case.stdin_input {
            cmd.stdin(Stdio::piped());
            let mut child = cmd
                .spawn()
                .context("Failed to spawn test process")?;

            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                stdin.write_all(stdin_data.as_bytes())?;
                drop(stdin);
            }

            let output = child
                .wait_with_output()
                .context("Failed to wait for test process")?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);

            let passed = exit_code == test_case.expected_exit_code
                && check_stdout_pattern(&stdout, test_case.expected_stdout_pattern.as_deref());

            return Ok(SmokeTestOutcome {
                test_name: test_case.name.clone(),
                passed,
                exit_code,
                stdout,
                stderr,
                duration_ms: start.elapsed().as_millis() as u64,
                error: None,
            });
        }

        let output = cmd
            .output()
            .context("Failed to execute test command")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        let passed = exit_code == test_case.expected_exit_code
            && check_stdout_pattern(&stdout, test_case.expected_stdout_pattern.as_deref());

        Ok(SmokeTestOutcome {
            test_name: test_case.name.clone(),
            passed,
            exit_code,
            stdout,
            stderr,
            duration_ms: start.elapsed().as_millis() as u64,
            error: None,
        })
    }

    pub fn quick_verify(binary_path: &str) -> Result<bool> {
        let path = Path::new(binary_path);
        if !path.exists() {
            return Ok(false);
        }

        let output = Command::new(binary_path)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("Failed to execute version check")?;

        Ok(output.status.success())
    }
}

fn check_stdout_pattern(stdout: &str, pattern: Option<&str>) -> bool {
    match pattern {
        Some(pat) => stdout.contains(pat),
        None => true,
    }
}
