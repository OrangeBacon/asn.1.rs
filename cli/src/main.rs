mod error;

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
    time::Instant,
};

use asn1::{AsnCompiler, Diagnostic};
use clap::{Parser, ValueEnum, ValueHint};
use error::{to_error, AsnCompilerCache};

#[derive(Parser)]
#[command(version, about)]
#[command(name = "asn1rs")]
#[clap(color = concolor_clap::color_choice())]
struct Cli {
    /// All initial source files to be parsed
    #[arg(required = true, value_hint = ValueHint::FilePath)]
    files: Vec<PathBuf>,

    /// Path to the output file. If '-' is passed, uses standard output.
    #[arg(short, long, default_value = "-")]
    output: PathBuf,

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

    /// Error message colour control
    #[command(flatten)]
    color: concolor_clap::Color,
}

#[derive(ValueEnum, Clone, Copy)]
enum Feature {
    /// Allow both upper and lowercase keywords.
    LowercaseKeywords,

    /// Allow non-ascii characters in identifiers
    UnicodeIdentifiers,

    /// Allow further whitespace characters
    UnicodeWhitespace,
}

fn main() -> ExitCode {
    let mut compiler = AsnCompiler::new();

    let Ok(result) = run(&mut compiler) else {
        return ExitCode::FAILURE;
    };

    for diag in &result {
        let err = to_error(diag).and_then(|r| Ok(r.eprint(AsnCompilerCache::new(&compiler))?));
        if let Err(err) = err {
            eprintln!("Error while printing error messages: {err:?}");
            return ExitCode::FAILURE;
        }
    }

    if result.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run(compiler: &mut AsnCompiler) -> Result<Vec<Diagnostic>, ExitCode> {
    let mut errors = vec![];

    let cli = Cli::parse();

    let features = if cli.strict {
        &[][..]
    } else {
        &[
            Feature::LowercaseKeywords,
            Feature::UnicodeIdentifiers,
            Feature::UnicodeWhitespace,
        ]
    };
    for feature in features.iter().copied().chain(cli.feature) {
        match feature {
            Feature::LowercaseKeywords => compiler.lowercase_keywords = true,
            Feature::UnicodeIdentifiers => compiler.unicode_identifiers = true,
            Feature::UnicodeWhitespace => compiler.unicode_whitespace = true,
        }
    }

    let mut timings = vec![];

    for path in cli.files {
        let Ok(source) = std::fs::read_to_string(&path) else {
            eprintln!("Unable to open source file `{path:?}`");
            return Err(ExitCode::FAILURE);
        };
        let display_name = path.to_string_lossy().to_string();

        let start = Instant::now();
        let res = compiler.add_file(display_name.clone(), source);
        let end = start.elapsed();

        timings.push(format!("Parse `{display_name}`: {end:?}"));

        match res {
            Ok(t) => {
                if cli.print_cst {
                    print!("{}", compiler.print_cst(t))
                }
            }
            Err(e) => errors.push(e),
        }
    }

    if !errors.is_empty() {
        return Ok(errors);
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

    if an.errors.is_empty() {
        let start = Instant::now();
        let code = an.rust_codegen();
        let end = start.elapsed();
        timings.push(format!("Codegen: {end:?}"));

        match code {
            Ok(s) => {
                if cli.output == Path::new("-") {
                    println!("{s}");
                } else if std::fs::write(cli.output, s).is_err() {
                    eprintln!("Error writing output file");
                }
            }
            Err(e) => eprintln!("{e:?}"),
        }
    }

    if cli.timing {
        for line in timings {
            eprintln!("{line}");
        }
    }

    Ok(vec![])
}
