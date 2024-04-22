use lexer::Lexer;

mod lexer;
mod token;

pub fn ada() {
    let source = r#"with Text_IO; use Text_IO;
    procedure hello is
    begin
       Put_Line("Hello world!");
    end hello;"#;
    let tokens = Lexer::run(source);

    println!("{tokens:?}");
}
