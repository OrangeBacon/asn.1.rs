use asn1::{lexer::Lexer, parser::Parser};

fn main() {
    let lexer = Lexer::new(
        0,
        "MyModule DEFINITIONS
        HELLO INSTRUCTIONS
        AUTOMATIC TAGS
        EXTENSIBILITY IMPLIED ::= BEGIN
            Hello ::= BOOLEAN
            world BOOLEAN /*hello */::= TRUE
            stuff BOOLEAN ::= FALSE
        END -- hi --",
    );

    let parser = Parser::new(lexer);

    match parser.run() {
        Ok(t) => println!("{t}"),
        Err(e) => println!("{e:?}"),
    }
}
