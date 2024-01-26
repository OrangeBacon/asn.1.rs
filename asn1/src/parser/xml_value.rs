use crate::{cst::Asn1Tag, token::TokenKind};

use super::{Parser, Result};

/// What kind of state is the XML parser currently in
#[derive(Debug, Clone, Copy)]
enum XMLState {
    /// Parsing the inside of an opening XML tag or a self closing tag
    OpenTag,

    /// Parsing the data between an opening and closing tag
    Data,

    /// Parsing the inside of a closing XML tag
    CloseTag,
}

impl<'a> Parser<'a> {
    /// Parse an XML value by matching XML tags and skipping over the content.
    /// Does not attempt to work out the type or value contained within the
    /// XML value, only its bounds.
    pub(super) fn xml_value(&mut self) -> Result {
        // skip leading whitespace and comments
        self.peek(&[])?;
        self.consume_comments();

        let mut state = vec![XMLState::OpenTag];
        while let Some(current) = state.last() {
            match current {
                XMLState::OpenTag => self.xml_open_tag(&mut state)?,
                XMLState::Data => self.xml_data(&mut state)?,
                XMLState::CloseTag => self.xml_close_tag(&mut state)?,
            }
        }

        Ok(())
    }

    /// Parse the inside of an XML tag
    fn xml_open_tag(&mut self, state: &mut Vec<XMLState>) -> Result {
        self.start_temp_vec(Asn1Tag::XMLValue)?;
        self.start_temp_vec(Asn1Tag::XMLTag)?;

        self.next_xml(&[TokenKind::Less])?;

        loop {
            let tok = self.next_xml(&[
                TokenKind::XMLData,
                TokenKind::Greater,
                TokenKind::XMLSingleTagEnd,
            ])?;

            if tok.kind == TokenKind::Greater {
                state.pop();
                state.push(XMLState::Data);
                self.end_temp_vec(Asn1Tag::XMLTag);
                self.start_temp_vec(Asn1Tag::XMLData)?;
                break;
            } else if tok.kind == TokenKind::XMLSingleTagEnd {
                state.pop();
                self.end_temp_vec(Asn1Tag::XMLTag);
                self.end_temp_vec(Asn1Tag::XMLValue);
                break;
            }
        }

        Ok(())
    }

    /// Parse the data within an XML element
    fn xml_data(&mut self, state: &mut Vec<XMLState>) -> Result {
        loop {
            let tok =
                self.peek_xml(&[TokenKind::XMLData, TokenKind::Less, TokenKind::XMLEndTag])?;

            if tok.kind == TokenKind::Less {
                state.push(XMLState::OpenTag);
                break;
            } else if tok.kind == TokenKind::XMLEndTag {
                state.pop();
                state.push(XMLState::CloseTag);
                break;
            }
            self.next_xml(&[])?;
        }
        Ok(())
    }

    /// Parse the inside of an XML closing tag
    fn xml_close_tag(&mut self, state: &mut Vec<XMLState>) -> Result {
        self.end_temp_vec(Asn1Tag::XMLData);
        self.start_temp_vec(Asn1Tag::XMLTag)?;

        self.next_xml(&[TokenKind::XMLEndTag])?;

        loop {
            let tok = self.next_xml(&[TokenKind::XMLData, TokenKind::Greater])?;

            if tok.kind == TokenKind::Greater {
                state.pop();
                break;
            }
        }

        self.end_temp_vec(Asn1Tag::XMLTag);
        self.end_temp_vec(Asn1Tag::XMLValue);
        Ok(())
    }
}
