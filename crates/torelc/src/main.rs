use std::{fs, path::PathBuf};

use clap::{Parser, ValueEnum};
use torel_lexer::lex;

#[derive(Debug, Parser)]
#[command(name = "torelc", about = "Torel compiler")]
struct Cli {
    /// Source file to compile.
    input: PathBuf,

    /// What compilation artifact to emit.
    #[arg(long, value_enum, default_value_t = Emit::Check)]
    emit: Emit,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Emit {
    Check,
    Tokens,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let source = fs::read_to_string(&cli.input)?;

    match cli.emit {
        Emit::Check => {
            let token_count = lex(&source).len();
            println!("checked {} token(s)", token_count);
        }
        Emit::Tokens => {
            for token in lex(&source) {
                println!("{:?} {:?}", token.span, token.kind);
            }
        }
    }

    Ok(())
}
