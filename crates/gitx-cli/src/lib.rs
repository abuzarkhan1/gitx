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
///
/// 0 success · 1 general error · 2 invalid arguments · 3 repository not found
/// · 4 not a Git repository · 5 index unavailable/corrupt · 6 unsupported
/// operation · 7 analysis incomplete.
fn exit_code_for(err: &anyhow::Error) -> i32 {
    let text = format!("{err:#}").to_lowercase();
    if text.contains("not inside a git repository") || text.contains("not a git repository") {
        4
    } else if text.contains("index unavailable")
        || text.contains("index corrupt")
        || text.contains("corrupt index")
        || (text.contains("index") && text.contains("schema"))
    {
        5
    } else if text.contains("unsupported") || text.contains("not supported") {
        6
    } else if text.contains("analysis incomplete") {
        7
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
        assert_eq!(exit_code_for(&anyhow!("unexpected failure")), 1);
    }

    #[test]
    fn maps_index_errors_to_5() {
        assert_eq!(
            exit_code_for(&anyhow!("index corrupt: cannot read commits")),
            5
        );
        assert_eq!(
            exit_code_for(&anyhow!("index schema is newer than this build")),
            5
        );
    }

    #[test]
    fn maps_unsupported_to_6() {
        assert_eq!(
            exit_code_for(&anyhow!("unsupported operation: blame on binary")),
            6
        );
    }

    #[test]
    fn maps_incomplete_analysis_to_7() {
        assert_eq!(
            exit_code_for(&anyhow!("analysis incomplete: unparsed manifests skipped")),
            7
        );
    }
}
