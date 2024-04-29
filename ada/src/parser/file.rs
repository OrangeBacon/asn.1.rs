use crate::{cst::TreeKind, token::TokenKind};

use super::Parser;

/// Parse a whole source file
pub fn file(p: &mut Parser) {
    let m = p.open();

    while !p.eof() {
        if p.at(TokenKind::KwPragma) {
            pragma(p);
        } else {
            p.advance_with_error("Expected a pragma :3");
        }
    }

    p.close(m, TreeKind::File);
}

/// Parse a pragma
fn pragma(p: &mut Parser) {
    assert!(p.at(TokenKind::KwPragma));
    let m = p.open();

    p.expect(TokenKind::KwPragma);
    p.expect(TokenKind::Identifier);

    if p.at(TokenKind::LParen) {
        pragma_arg_list(p);
    }
    p.expect(TokenKind::SemiColon);

    p.close(m, TreeKind::Pragma);
}

/// List of arguments to a pragma
fn pragma_arg_list(p: &mut Parser) {
    assert!(p.at(TokenKind::LParen));
    let m = p.open();

    p.expect(TokenKind::LParen);
    while !p.at(TokenKind::RParen) && !p.at(TokenKind::Eof) {
        if p.at(TokenKind::Identifier) {
            pragma_arg(p);
        } else {
            break;
        }
    }
    p.expect(TokenKind::RParen);

    p.close(m, TreeKind::PragmaArgumentList);
}

/// A single argument to a pragma
fn pragma_arg(p: &mut Parser) {
    assert!(p.at(TokenKind::Identifier));
    let m = p.open();

    p.expect(TokenKind::Identifier);
    if p.eat(TokenKind::Arrow) {
        p.expect(TokenKind::Identifier);
    }

    if !p.at(TokenKind::RParen) {
        p.expect(TokenKind::Comma);
    }

    p.close(m, TreeKind::PragmaArgument);
}
