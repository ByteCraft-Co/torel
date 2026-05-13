use std::{fs, path::PathBuf};

use clap::{Parser, ValueEnum};
use torel_backend::{Backend, BackendTarget};
use torel_codegen::{CodegenTarget, codegen};
use torel_codegen_llvm::LlvmBackend;
use torel_diagnostics::FileId;
use torel_effects::{check_effects, check_failures};
use torel_ir::lower_ast;
use torel_lexer::lex;
use torel_mir::lower_to_mir;
use torel_ownership::check_ownership;
use torel_parse::parse_source_file;
use torel_session::{SourceFile, render_diagnostic};
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
    Typed,
    Mir,
    LlvmIr,
}

fn main() {
    if let Err(err) = run() {
        eprint!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let source = fs::read_to_string(&cli.input).map_err(format_error)?;
    let source_file = SourceFile::new(FileId(0), cli.input.clone(), source);
    let tokens = lex(&source_file.text);

    match cli.emit {
        Emit::Tokens => {
            for token in tokens {
                println!("{:?} {:?}", token.span, token.kind);
            }
        }
        Emit::Check | Emit::Ast | Emit::Hir | Emit::Typed | Emit::Mir | Emit::LlvmIr => {
            let ast = parse_source_file(&tokens)
                .map_err(|err| render_diagnostic(&source_file, &err.into_diagnostic()))?;

            if matches!(cli.emit, Emit::Ast) {
                println!("{ast:#?}");
                return Ok(());
            }

            let hir = lower_ast(&ast);

            if matches!(cli.emit, Emit::Hir) {
                println!("{hir:#?}");
                return Ok(());
            }

            let typed = check_types(&hir)
                .map_err(|err| render_diagnostic(&source_file, &err.to_diagnostic()))?;

            if matches!(cli.emit, Emit::Typed) {
                println!("{typed:#?}");
                return Ok(());
            }

            let effects = check_effects(&typed).map_err(format_error)?;
            let failures = check_failures(&typed).map_err(format_error)?;
            let ownership = check_ownership(&typed).map_err(format_error)?;
            let mir = lower_to_mir(&typed)
                .map_err(|err| render_diagnostic(&source_file, &err.to_diagnostic()))?;

            if matches!(cli.emit, Emit::Mir) {
                print!("{}", mir.pretty());
                return Ok(());
            }

            if matches!(cli.emit, Emit::LlvmIr) {
                let output = LlvmBackend
                    .emit(&mir, BackendTarget::LlvmIr)
                    .map_err(format_error)?;

                if let Some(text) = output.text {
                    print!("{text}");
                }

                return Ok(());
            }

            let output = codegen(&typed, CodegenTarget::CheckOnly).map_err(format_error)?;

            println!("{}", output.text);
            println!(
                "pipeline: source -> lexer -> parser -> AST -> HIR -> name resolution -> typed IR -> type/return checks -> effect/failure/ownership checks -> MIR -> MIR validation -> codegen"
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

fn format_error(err: impl std::fmt::Display) -> String {
    format!("error: {err}\n")
}
