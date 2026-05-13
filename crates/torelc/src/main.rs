use std::{fs, path::PathBuf};

use clap::{Parser, ValueEnum};
use torel_codegen::{CodegenTarget, codegen};
use torel_effects::{check_effects, check_failures};
use torel_ir::lower_ast;
use torel_lexer::lex;
use torel_ownership::check_ownership;
use torel_parse::parse_source_file;
use torel_typeck::check_types;

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
    Ast,
    Hir,
    LlvmIr,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let source = fs::read_to_string(&cli.input)?;
    let tokens = lex(&source);

    match cli.emit {
        Emit::Tokens => {
            for token in tokens {
                println!("{:?} {:?}", token.span, token.kind);
            }
        }
        Emit::Check | Emit::Ast | Emit::Hir | Emit::LlvmIr => {
            let ast = parse_source_file(&tokens)?;

            if matches!(cli.emit, Emit::Ast) {
                println!("{ast:#?}");
                return Ok(());
            }

            let hir = lower_ast(&ast);

            if matches!(cli.emit, Emit::Hir) {
                println!("{hir:#?}");
                return Ok(());
            }

            let typed = check_types(&hir)?;
            let effects = check_effects(&typed)?;
            let failures = check_failures(&typed)?;
            let ownership = check_ownership(&typed)?;
            let target = match cli.emit {
                Emit::LlvmIr => CodegenTarget::LlvmIr,
                _ => CodegenTarget::CheckOnly,
            };
            let output = codegen(&typed, target)?;

            println!("{}", output.text);
            println!(
                "pipeline: source -> lexer -> parser -> AST -> HIR -> name resolution -> typed IR -> type/return checks -> effect/failure/ownership checks -> codegen"
            );
            println!(
                "checks: types={} effects={} failures={} ownership_regions={}",
                typed.procs.len(),
                effects.checked_effect_sets,
                failures.checked_failure_sets,
                ownership.checked_owner_regions
            );
        }
    }

    Ok(())
}
