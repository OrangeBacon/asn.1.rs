use asn1::{lexer::Lexer, parser::Parser};

fn main() {
    let lexer = Lexer::new(
        0,
        "Hello ::= BOOLEAN",
    );

    let parser = Parser::new(lexer);

    println!("{:?}", parser.run());
}
