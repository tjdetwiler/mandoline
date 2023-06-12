use clap::{Parser, Subcommand};

#[derive(clap::Args, Debug)]
pub struct SvgArgs {
    pub stl_path: String,

    /// Output path for svg files.
    #[arg(short, long)]
    pub output: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Svg(SvgArgs),
}
