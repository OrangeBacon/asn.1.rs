use asn1::lexer::Lexer;

fn main() {
    let lexer = Lexer::new(0, "{',;=@[]}");

    for tok in lexer {
        println!("{:?}", tok);
    }
}
