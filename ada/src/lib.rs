use lexer::Lexer;

use crate::parser::Parser;

mod cst;
mod diagnostic;
mod lexer;
mod parser;
mod token;

/// Root of an ada compiler
pub struct Compiler;

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    /// Create a new compiler
    pub fn new() -> Compiler {
        Compiler
    }

    pub fn add_file(&mut self, source: String) {
        let parse = Parser::run(Lexer::run(&source));

        println!("{parse:?}");
    }
}
