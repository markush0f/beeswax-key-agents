use clap::Parser;

/// Agent Key Detector CLI
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory or file to scan for keys
    #[arg(short, long)]
    path: String,
}

fn main() {
    let args = Args::parse();

    println!("Starting analysis on path: {}", args.path);
}
