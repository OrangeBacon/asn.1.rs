use asn1::lexer::Lexer;

fn main() {
    let lexer = Lexer::new(0, "{...::=:= -- hello --}--world\n[]}--a");

    for tok in lexer {
        println!("{:?}", tok);
    }
}
