use asn1::{
    lexer::{Lexer, LexerError},
    parser::Parser,
};

fn main() {
    let source = r#"
    MyModule { ident(5) } "/1/2" DEFINITIONS
    HELLO INSTRUCTIONS
    AUTOMATIC TAGS
    EXTENSIBILITY IMPLIED ::= BEGIN
        EXPORTS a, b, c {};

        HELLO ::= BOOLEAN
        world BOOLEAN /*hello */::= TRUE
        stuff BOOLEAN ::= FALSE
        MyStuff ::= NULL
        foo NULL ::= NULL
        Iri ::= OID-IRI
        enc OID-IRI ::= "/ISO/Registration_Authority/19785.CBEFF/Organizations/JTC1-SC37/Patron-formats/TLV-encoded"
        MyInt ::= INTEGER { a(-5), b(Module.num) }
        a INTEGER ::= a -- comment
        b INTEGER --comment2--::= -3
        a ::= <INTEGER/>
        b ::= <Hello></Hello>
        c ::= <World><true/></World>
        d ::= <Foo>-5</Foo>
        e ::= <I>/a/b</I>
        F ::= ENUMERATED {
            a(5), b, c(7), ... !-5, z(3)
        }
        G ::= OBJECT IDENTIFIER
        h OBJECT IDENTIFIER ::= { iso standard 8571 application-context(1) }
        h OBJECT IDENTIFIER ::= { 1 0 8571 1 }
        i OBJECT IDENTIFIER ::= { A.b c(D.e) }
    END -- hi --"#;
    let lexer = Lexer::new(0, source);

    let parser = Parser::new(lexer);

    match parser.run() {
        Ok(t) => println!("{t}"),
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
