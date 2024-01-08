use asn1::{
    lexer::{Lexer, LexerError},
    parser::Parser,
};

fn main() {
    let source = std::fs::read_to_string("test/foo.asn1").unwrap();

    let lexer = Lexer::new(0, &source);

    let parser = Parser::new(lexer);

    match parser.run() {
        Ok(t) => print!("{t}"),
        Err(LexerError::Expected { kind, offset, .. }) => {
            let at = offset
                .map(|o| {
                    let s: String = source[o..].chars().take(15).collect();
                    format!("{s:?}")
                })
                .unwrap_or("EOF".to_string());

            println!("Expected {{ kind: {kind:?}, at: {at} }}");
        }
        Err(e) => println!("{e:?}"),
    }
}
