use clap::{Parser, ValueEnum};

/// CLI struct to parse arguments
#[derive(Debug, Clone, Parser)]
#[clap(author, version, about)]
pub struct CLI {
    /// List of words to calculate the values of
    pub words: Vec<String>,

    /// Fast Mode
    #[arg(short = 'f', long = "fast")]
    pub fast: bool,

    /// Recursive Mode
    #[arg(short = 'r', long = "recursive")]
    pub recursive: bool,

    /// Limit output verbosity (e.g., omit individual letter values)
    #[arg(short = 'l', long = "less")]
    pub less: bool,

    /// Eliminate formatting and extra text (simple output)
    #[arg(short = 'R', long = "raw")]
    pub raw: bool,

    /// Exclude total overall value from the output
    #[arg(long = "no-total")]
    pub no_total: bool,

    /// Format the output as json
    #[arg(long = "json")]
    pub json: bool,

    /// Print out the table used to determine the values
    #[arg(long = "table")]
    pub table: bool,

    /// Whether to print output with color
    #[arg(long)]
    #[clap(value_enum, default_value_t=Color::Auto)]
    pub color: Color,

    /// Whether to print output with decorations (bold, italic, etc.)
    #[arg(long)]
    #[clap(value_enum, default_value_t=Decorations::Auto)]
    pub decorations: Decorations,

    /// Silence Extra Output (such as "Note:")
    #[arg(short, long)]
    pub quiet: bool,
}

/// Color Enum for Color Choice
#[derive(Debug, Clone, ValueEnum)]
pub enum Color {
    Auto,
    Always,
    Never,
}

/// Decorations enum for bold, italics, etc.
#[derive(Debug, Clone, ValueEnum)]
pub enum Decorations {
    Auto,
    Always,
    Never,
}
