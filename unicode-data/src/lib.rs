#![allow(clippy::all, unused)]

mod gc;
mod whitespace;
mod xid;

pub use gc::{
    CONTROL, FORMAT, LINE_SEPARATOR, LOWERCASE_LETTER, PARAGRAPH_SEPARATOR, PRIVATE_USE, SURROGATE,
    UPPERCASE_LETTER, SPACE_SEPARATOR
};
pub use whitespace::WHITE_SPACE;
pub use xid::{XID_CONTINUE, XID_START};
