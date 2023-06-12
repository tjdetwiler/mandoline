use crate::args::Subcommand;
use clap::Parser;

mod args;
mod svg;

fn main() {
    let args = args::Args::parse();
    match args.command {
        args::Commands::Svg(svg) => svg::SvgCommand::run_command(svg),
    }
}
