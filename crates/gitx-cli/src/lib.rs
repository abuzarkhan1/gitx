pub mod cli;
pub mod commands;

pub use cli::{Cli, Commands};

/// Execute a parsed `gitx` invocation. Returns the process exit code.
pub fn run(cli: Cli) -> i32 {
    match commands::dispatch(cli) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("gitx: {err:#}");
            exit_code_for(&err)
        }
    }
}

/// Map common failures to the documented exit codes (docs/07 §19).
fn exit_code_for(err: &anyhow::Error) -> i32 {
    let text = format!("{err:#}").to_lowercase();
    if text.contains("not inside a git repository") || text.contains("not a git repository") {
        4
    } else if text.contains("no such branch")
        || text.contains("no such tag")
        || text.contains("does not exist")
        || text.contains("ambiguous")
        || text.contains("invalid object id")
    {
        2
    } else if text.contains("no commits") || text.contains("repository has no commits") {
        3
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::exit_code_for;
    use anyhow::anyhow;

    #[test]
    fn maps_not_a_repo_to_4() {
        assert_eq!(
            exit_code_for(&anyhow!("not inside a Git repository (use --repo <PATH>)")),
            4
        );
    }

    #[test]
    fn maps_invalid_argument_to_2() {
        assert_eq!(exit_code_for(&anyhow!("no such branch `x`")), 2);
    }

    #[test]
    fn maps_general_errors_to_1() {
        assert_eq!(exit_code_for(&anyhow!("index corrupt")), 1);
    }
}
