use clap::{Parser, ValueEnum};

#[derive(Clone, Debug, Parser, ValueEnum)]
pub enum ShellType {
    Bash,
    Fish,
}

#[derive(Debug, Parser)]
#[command(author, about, long_about = None, version)]
pub struct Args {
    #[arg(verbatim_doc_comment)]
    /// Type of the shell history
    ///   Example: sh-history-filter --shell-type bash --history-text '~/.bash_history'
    ///     *
    #[arg(long)]
    #[arg(value_enum)]
    #[arg(visible_alias = "sh")]
    #[arg(value_name = "TYPE")]
    pub shell: ShellType,

    #[arg(verbatim_doc_comment)]
    /// History text to apply filter
    ///   Example: sh-history-filter --shell-type bash --history-path '~/.bash_history'
    #[arg(long)]
    #[arg(value_name = "PATH")]
    pub history_path: String,
}
