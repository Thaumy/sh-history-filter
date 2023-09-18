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
    /// History text to apply filter
    ///   Example: sh-history-filter --shell-type bash --history-text 'echo hi'
    #[arg(long)]
    #[arg(value_name = "TEXT")]
    pub history_text: String,

    #[arg(verbatim_doc_comment)]
    /// Type of the shell history
    ///   Example: sh-history-filter --shell-type bash --history-text 'echo hi'
    ///     *
    #[arg(long)]
    #[arg(value_enum)]
    #[arg(visible_alias = "sh-type")]
    #[arg(value_name = "TYPE")]
    pub shell_type: ShellType,
}
