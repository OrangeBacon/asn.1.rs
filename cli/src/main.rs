use asn1::{
    lexer::{Lexer, LexerError},
    parser::Parser,
};

fn main() {
    let types = ["MyRef.Foo", "MyTypeRef", "a < INTEGER"];

    for source in types {
        let lexer = Lexer::new(0, source);

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
}
