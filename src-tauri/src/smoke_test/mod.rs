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
                    name: "version_check".to_string(),
                    command: binary_path.to_string(),
                    args: vec!["--version".to_string()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: Some(format!("Python {}", version)),
                },
                TestCase {
                    name: "import_sys".to_string(),
                    command: binary_path.to_string(),
                    args: vec![
                        "-c".to_string(),
                        "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')"
                            .to_string(),
                    ],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: None,
                },
            ],
            "node" | "nodejs" => vec![
                TestCase {
                    name: "version_check".to_string(),
                    command: binary_path.to_string(),
                    args: vec!["--version".to_string()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: Some(format!("v{}", version)),
                },
                TestCase {
                    name: "eval_expression".to_string(),
                    command: binary_path.to_string(),
                    args: vec!["-e".to_string(), "console.log(2+2)".to_string()],
                    expected_exit_code: 0,
                    timeout_secs: 10,
                    stdin_input: None,
                    expected_stdout_pattern: Some("4".to_string()),
                },
            ],
            "rust" | "rustc" => vec![TestCase {
                name: "version_check".to_string(),
                command: binary_path.to_string(),
                args: vec!["--version".to_string()],
                expected_exit_code: 0,
                timeout_secs: 10,
                stdin_input: None,
                expected_stdout_pattern: None,
            }],
            "go" | "golang" => vec![TestCase {
                name: "version_check".to_string(),
                command: binary_path.to_string(),
                args: vec!["version".to_string()],
                expected_exit_code: 0,
                timeout_secs: 10,
                stdin_input: None,
                expected_stdout_pattern: None,
            }],
            "java" | "jdk" => vec![TestCase {
                name: "version_check".to_string(),
                command: binary_path.to_string(),
                args: vec!["-version".to_string()],
                expected_exit_code: 0,
                timeout_secs: 10,
                stdin_input: None,
                expected_stdout_pattern: None,
            }],
            "gcc" | "g++" | "cpp" | "c" => vec![TestCase {
                name: "version_check".to_string(),
                command: binary_path.to_string(),
                args: vec!["--version".to_string()],
                expected_exit_code: 0,
                timeout_secs: 10,
                stdin_input: None,
                expected_stdout_pattern: None,
            }],
            _ => vec![TestCase {
                name: "version_check".to_string(),
                command: binary_path.to_string(),
                args: vec!["--version".to_string()],
                expected_exit_code: 0,
                timeout_secs: 10,
                stdin_input: None,
                expected_stdout_pattern: None,
            }],
        };

        Self {
            language: language.to_string(),
            version: version.to_string(),
            binary_path: binary_path.to_string(),
            test_cases,
        }
    }
}
