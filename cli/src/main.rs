use std::{path::PathBuf, time::Instant};

use asn1::{AsnCompiler, ParserError};
use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
#[command(name = "asn1rs")]
struct Cli {
    /// All initial source files to be parsed
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Display timing information for various phases within the compiler
    #[arg(short, long)]
    timing: bool,

    /// Display the parsed Concrete Syntax Tree of each file
    #[arg(long)]
    print_cst: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut compiler = AsnCompiler::new();

    let mut timings = vec![];

    for path in cli.files {
        let source = std::fs::read_to_string(&path).unwrap();
        let display_name = path.to_string_lossy().to_string();

        let start = Instant::now();
        let res = compiler.add_file(display_name.clone(), source.clone());
        let end = start.elapsed();

        timings.push(format!("Parse `{display_name}`: {end:?}"));

        match res {
            Ok(t) => {
                if cli.print_cst {
                    print!("{}", compiler.print_cst(t))
                }
            }
            Err(
                | ref err @ ParserError::Expected { offset, .. }
                | ref err @ ParserError::TypeValueError { offset, .. },
            ) => {
                let at: String = source[offset..].chars().take(15).collect();

                eprintln!("{err:?} = {at:?}");
            }
            Err(e) => println!("{e:?}"),
        }
    }

    let start = Instant::now();
    let an = compiler.analysis();
    let end = start.elapsed();
    timings.push(format!("Analysis: {end:?}"));

    if !an.errors.is_empty() {
        eprintln!("{:?}", an.errors);
    }

    if !an.warnings.is_empty() {
        eprintln!("{:?}", an.warnings);
    }

    if cli.timing {
        for line in timings {
            println!("{line}");
        }
    }
}
