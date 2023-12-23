use asn1::{lexer::Lexer, parser::Parser};

fn main() {
    let lexer = Lexer::new(
        0,
        "Hello ::= BOOLEAN world BOOLEAN /*hello */::= TRUE stuff BOOLEAN ::= FALSE -- hi --",
    );

    let parser = Parser::new(lexer);

    match parser.run() {
        Ok(t) => println!("{t}"),
        Err(e) => println!("{e:?}"),
    }
}
