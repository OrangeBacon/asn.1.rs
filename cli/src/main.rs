use asn1::lexer::Lexer;

fn main() {
    let lexer = Lexer::new(
        0,
        "{...::=:= -- hello --}--world\n[]}/*a*/{}
    /*/*abc*/aa/*aa*/*/;;! hello hi-world a-57 a-;",
    );

    for tok in lexer {
        println!("{:?}", tok);
    }
}
