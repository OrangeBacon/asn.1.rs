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

    match parser.run() {
        Ok(t) => print!("{t}"),
        Err(ParserError::Expected { kind, offset, .. }) => {
            let at: String = source[offset..].chars().take(15).collect();

            println!("Expected {{ kind: {kind:?}, at: {at:?} }}");
        }
        Err(e) => println!("{e:?}"),
    }
}
