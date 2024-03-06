use std::{path::PathBuf, time::Instant};

use asn1::{AsnCompiler, ParserError};
use clap::{Parser, ValueEnum, ValueHint};

#[derive(Parser)]
#[command(version, about)]
#[command(name = "asn1rs")]
struct Cli {
    /// All initial source files to be parsed
    #[arg(required = true, value_hint = ValueHint::FilePath)]
    files: Vec<PathBuf>,

    /// Display timing information for various phases within the compiler
    #[arg(short, long)]
    timing: bool,

    /// Display the parsed Concrete Syntax Tree of each file
    #[arg(long)]
    print_cst: bool,

    /// Disable all default features.
    #[arg(short, long)]
    strict: bool,

    /// Enable any additional feature
    #[arg(value_enum, short, long)]
    feature: Vec<Feature>,
}

#[derive(ValueEnum, Clone, Copy)]
enum Feature {
    /// Perform case-folding before matching any keywords.
    IgnoreKeywordCase,

    /// Allow non-ascii characters in identifiers
    UnicodeIdentifiers,

    /// Allow further whitespace characters
    UnicodeWhitespace,
}

fn main() {
    let cli = Cli::parse();

    let mut compiler = AsnCompiler::new();

    let features = if cli.strict {
        &[][..]
    } else {
        &[
            Feature::IgnoreKeywordCase,
            Feature::UnicodeIdentifiers,
            Feature::UnicodeWhitespace,
        ]
    };
    for feature in features.iter().copied().chain(cli.feature) {
        match feature {
            Feature::IgnoreKeywordCase => compiler.ignore_keyword_case = true,
            Feature::UnicodeIdentifiers => compiler.unicode_identifiers = true,
            Feature::UnicodeWhitespace => compiler.unicode_whitespace = true,
        }
    }

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
                ref err @ ParserError::Expected { offset, .. }
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
