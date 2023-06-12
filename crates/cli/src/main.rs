use clap::Parser;

mod args;
mod svg;

fn main() {
    let args = args::Args::parse();
    match args.command {
        args::Commands::Svg(svg_args) => svg::svg_command(svg_args),
    }
}
