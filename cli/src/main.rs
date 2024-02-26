use std::time::Instant;

use asn1::{AsnCompiler, ParserError};

fn main() {
    let mut compiler = AsnCompiler::new();

    let path = "test/typed.asn1";
    let source = std::fs::read_to_string(path).unwrap();

    let start = Instant::now();
    let res = compiler.add_file(path.to_string(), source.clone());
    let end = start.elapsed();

    match res {
        Ok(t) => print!("{}", compiler.print_cst(t)),
        Err(
            ref err @ (ParserError::Expected { offset, .. }
            | ParserError::TypeValueError { offset, .. }),
        ) => {
            let at: String = source[offset..].chars().take(15).collect();

            println!("{err:?} = {at:?}");
        }
        Err(e) => println!("{e:?}"),
    }

    let an_start = Instant::now();
    let an = compiler.analysis();
    let an_end = an_start.elapsed();

    if !an.errors.is_empty() {
        println!("{:?}", an.errors);
    }

    if !an.warnings.is_empty() {
        println!("{:?}", an.warnings);
    }

    println!("Parse {end:?}");
    println!("Analysis {an_end:?}");
}
