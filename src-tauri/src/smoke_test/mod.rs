use serde::{Deserialize, Serialize};

pub mod runner;

pub use runner::SmokeTestRunner;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmokeTestConfig {
    pub language: String,
    pub version: String,
    pub binary_path: String,
    pub test_cases: Vec<TestCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub expected_exit_code: i32,
    pub timeout_secs: u64,
    pub stdin_input: Option<String>,
    pub expected_stdout_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmokeTestOutcome {
    pub test_name: String,
    pub passed: bool,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmokeTestSuiteResult {
    pub language: String,
    pub version: String,
    pub all_passed: bool,
    pub outcomes: Vec<SmokeTestOutcome>,
    pub total_duration_ms: u64,
}

impl SmokeTestConfig {
    pub fn default_for_language(language: &str, version: &str, binary_path: &str) -> Self {
        let test_cases = match language.to_lowercase().as_str() {
            "python" => vec![
                TestCase {
                    name: "version_check".into(),
                    command: binary_path.into(),
                    args: vec!["--version".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: Some("Python 3".into()),
                },
                TestCase {
                    name: "import_test".into(),
                    command: binary_path.into(),
                    args: vec!["-c".into(), "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: None,
                },
                TestCase {
                    name: "pip_check".into(),
                    command: binary_path.into(),
                    args: vec!["-m".into(), "pip".into(), "--version".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: None,
                },
            ],
            "node" | "nodejs" => vec![
                TestCase {
                    name: "version_check".into(),
                    command: binary_path.into(),
                    args: vec!["--version".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: Some("v".into()),
                },
                TestCase {
                    name: "eval_js".into(),
                    command: binary_path.into(),
                    args: vec!["-e".into(), "console.log(2+2)".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: Some("4".into()),
                },
                TestCase {
                    name: "npm_check".into(),
                    command: binary_path.into(),
                    args: vec!["-e".into(), "console.log(require('module').builtinModules.length > 0)".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: Some("true".into()),
                },
            ],
            "rust" | "rustc" => vec![
                TestCase {
                    name: "version_check".into(),
                    command: binary_path.into(),
                    args: vec!["--version".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: None,
                },
                TestCase {
                    name: "cargo_check".into(),
                    command: binary_path.replace("rustc", "cargo"),
                    args: vec!["--version".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: None,
                },
            ],
            "go" | "golang" => vec![
                TestCase {
                    name: "version_check".into(),
                    command: binary_path.into(),
                    args: vec!["version".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: None,
                },
                TestCase {
                    name: "env_check".into(),
                    command: binary_path.into(),
                    args: vec!["env".into(), "GOPATH".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: None,
                },
            ],
            "java" | "jdk" => vec![
                TestCase {
                    name: "version_check".into(),
                    command: binary_path.into(),
                    args: vec!["-version".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: None,
                },
            ],
            "gcc" | "g++" | "cpp" | "c" => vec![
                TestCase {
                    name: "version_check".into(),
                    command: binary_path.into(),
                    args: vec!["--version".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: None,
                },
            ],
            "ruby" => vec![
                TestCase {
                    name: "version_check".into(),
                    command: binary_path.into(),
                    args: vec!["--version".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: None,
                },
                TestCase {
                    name: "eval_ruby".into(),
                    command: binary_path.into(),
                    args: vec!["-e".into(), "puts 2+2".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: Some("4".into()),
                },
            ],
            "deno" => vec![
                TestCase {
                    name: "version_check".into(),
                    command: binary_path.into(),
                    args: vec!["--version".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: None,
                },
                TestCase {
                    name: "eval_ts".into(),
                    command: binary_path.into(),
                    args: vec!["eval".into(), "console.log(2+2)".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: Some("4".into()),
                },
            ],
            _ => vec![
                TestCase {
                    name: "version_check".into(),
                    command: binary_path.into(),
                    args: vec!["--version".into()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: None,
                },
            ],
        };

        Self { language: language.into(), version: version.into(), binary_path: binary_path.into(), test_cases }
    }
}
