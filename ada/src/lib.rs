use lexer::Lexer;

mod lexer;
mod token;

pub fn ada() {
    let source = "Hello, world";
    let tokens = Lexer::run(source);

    println!("{tokens:?}");
}
