#[derive(clap::Args, Debug)]
pub struct SvgArgs {
    /// Output path for svg files.
    #[arg(short, long)]
    pub output: String,

    #[arg(short, long)]
    pub layer: Option<usize>,

    pub stl_path: String,

    pub grid: bool,
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, clap::Subcommand)]
pub enum Commands {
    Svg(SvgArgs),
}

pub trait Subcommand<T: clap::Args> {
    fn run_command(args: T);
}
