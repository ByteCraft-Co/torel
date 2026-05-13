use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "torel", about = "Torel language toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print toolchain version information.
    Version,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Version => {
            println!("torel {}", env!("CARGO_PKG_VERSION"));
        }
    }
}
