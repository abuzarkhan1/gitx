use crate::cli::Cli;
use clap::CommandFactory;

/// Emit shell completions to stdout (docs/07 §20). Example:
/// `gitx completions bash > /usr/local/etc/bash_completion.d/gitx`
pub fn completions(shell: clap_complete::Shell) -> anyhow::Result<()> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
    Ok(())
}
