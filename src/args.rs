use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "rdiff",
    author = "Caleb Kornegay <caleb.kornegay@gmail.com>",
    version = "0.0.1",
    about = "A TUI app to visually diff two text files",
    long_about = "This tool shows a side-by-side diff of two or three files\nAuthor: Caleb Kornegay <caleb.kornegay@gmail.com>"
)]

pub struct Args {
    #[arg(help = "First file")]
    pub file_1: String,

    #[arg(help = "Second file")]
    pub file_2: String,

    #[arg(help = "Optional third file")]
    pub file_3: Option<String>,

    #[arg(short = 'x', long)]
    pub hex: bool,

    #[arg(short = 's', long)]
    pub suppress_common_lines: bool,

    #[arg(short = 'w', long)]
    pub width: Option<usize>,

    #[arg(short = 'c', long)]
    pub context_lines: Option<usize>,
}