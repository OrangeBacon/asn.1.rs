use asn1::lexer::Lexer;

fn main() {
    let lexer = Lexer::new(0, "source");

    for tok in lexer {
        println!("{:?}", tok);
    }
}
