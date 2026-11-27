use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum CliCommand {
    Help,
    Version,
    List,
    Use { tool: String, version: String },
    Doctor,
    RunGui,
}

pub fn parse_cli_args(args: &[String]) -> CliCommand {
    if args.len() <= 1 {
        return CliCommand::RunGui;
    }

    match args[1].as_str() {
        "--help" | "-h" | "help" => CliCommand::Help,
        "--version" | "-v" | "version" => CliCommand::Version,
        "list" | "ls" => CliCommand::List,
        "use" => {
            if args.len() >= 4 {
                CliCommand::Use {
                    tool: args[2].to_lowercase(),
                    version: args[3].clone(),
                }
            } else {
                CliCommand::Help
            }
        }
        "doctor" | "diagnose" => CliCommand::Doctor,
        _ => CliCommand::RunGui,
    }
}

pub fn get_devignite_root() -> PathBuf {
    if let Ok(custom_root) = std::env::var("DEVIGNITE_ROOT") {
        if !custom_root.trim().is_empty() {
            return PathBuf::from(custom_root);
        }
    }

    directories::UserDirs::new()
        .map(|dirs| dirs.home_dir().join(".devignite"))
        .unwrap_or_else(|| PathBuf::from(".devignite"))
}

pub fn print_help() {
    println!(
        r#"DevIgnite - High-Performance Sandboxed Developer Environment Manager

USAGE:
    devignite [COMMAND] [OPTIONS]

COMMANDS:
    list, ls                    List all installed runtimes
    use <tool> <version>        Switch active runtime symlink
    doctor, diagnose            Run integrity checks and diagnostics
    version, -v, --version      Display DevIgnite version
    help, -h, --help            Print help information

ENVIRONMENT:
    DEVIGNITE_ROOT              Custom directory for ~/.devignite storage
"#
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cli_args_help() {
        let args = vec!["devignite".into(), "--help".into()];
        assert_eq!(parse_cli_args(&args), CliCommand::Help);

        let args_short = vec!["devignite".into(), "-h".into()];
        assert_eq!(parse_cli_args(&args_short), CliCommand::Help);
    }

    #[test]
    fn test_parse_cli_args_list() {
        let args = vec!["devignite".into(), "list".into()];
        assert_eq!(parse_cli_args(&args), CliCommand::List);

        let args_ls = vec!["devignite".into(), "ls".into()];
        assert_eq!(parse_cli_args(&args_ls), CliCommand::List);
    }

    #[test]
    fn test_parse_cli_args_use() {
        let args = vec!["devignite".into(), "use".into(), "Node".into(), "20.11.0".into()];
        assert_eq!(
            parse_cli_args(&args),
            CliCommand::Use {
                tool: "node".into(),
                version: "20.11.0".into(),
            }
        );
    }

    #[test]
    fn test_parse_cli_args_empty_runs_gui() {
        let args = vec!["devignite".into()];
        assert_eq!(parse_cli_args(&args), CliCommand::RunGui);
    }

    #[test]
    fn test_get_devignite_root_default() {
        let root = get_devignite_root();
        assert!(root.to_string_lossy().contains(".devignite"));
    }
}
