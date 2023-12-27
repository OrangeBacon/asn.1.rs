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
        HELLO ::= BOOLEAN
        world BOOLEAN /*hello */::= TRUE
        stuff BOOLEAN ::= FALSE
        MyStuff ::= NULL
        foo NULL ::= NULL
        Iri ::= OID-IRI
        enc OID-IRI ::= "/ISO/Registration_Authority/19785.CBEFF/Organizations/JTC1-SC37/Patron-formats/TLV-encoded"
        MyInt ::= INTEGER { a(-5), b(Module.num) }
        a INTEGER ::= a
        b INTEGER ::= -3
        a ::= <INTEGER/>
        b ::= <Hello></Hello>
        c ::= <World><true/></World>
        d ::= <Foo>-5</Foo>
        e ::= <I>/a/b</I>
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
