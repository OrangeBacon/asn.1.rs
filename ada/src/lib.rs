use lexer::Lexer;

mod lexer;
mod token;

pub fn ada() {
    let source = r#"with Text_IO; use Text_IO;
    procedure hello is -- comment
    begin
       Put_Line("Hello "" world!" & 16#a.4#e-7);
    end hello;"#;
    let tokens = Lexer::run(source);

    println!("{tokens:?}");
}
