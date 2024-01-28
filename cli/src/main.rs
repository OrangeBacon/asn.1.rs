use std::time::Instant;

use asn1::{
    lexer::Lexer,
    parser::{Parser, ParserError},
};

fn main() {
    let source = std::fs::read_to_string("test/foo.asn1").unwrap();

    let mut lexer = Lexer::new(0, &source);

    if std::env::args().any(|a| a == "lex") {
        while let Ok(t) = lexer.next_token() {
            println!("{t:?}");
        }
        return;
    }

    let parser = Parser::new(lexer);

    let start = Instant::now();
    let res = parser.run();
    let end = start.elapsed();

    match res {
        Ok(t) => print!("{t}"),
        Err(
            ref err @ (ParserError::Expected { offset, .. }
            | ParserError::TypeValueError { offset, .. }),
        ) => {
            let at: String = source[offset..].chars().take(15).collect();

            println!("{err:?} = {at:?}");
        }
        Err(e) => println!("{e:?}"),
    }

    println!("Completed in {end:?}");
}
