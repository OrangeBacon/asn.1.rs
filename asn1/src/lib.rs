#![forbid(unsafe_code)]

mod analysis;
mod ast;
mod compiler;
mod cst;
mod lexer;
mod parser;
mod token;
mod util;

pub use compiler::AsnCompiler;
pub use parser::ParserError;

const _: () = assert!(
    unicode_normalization::UNICODE_VERSION.0 == 15
        && unicode_normalization::UNICODE_VERSION.1 == 1
        && unicode_normalization::UNICODE_VERSION.2 == 0,
    "Mismatched unicode normalisation version"
);
