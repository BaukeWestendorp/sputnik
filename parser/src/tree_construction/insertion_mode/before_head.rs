use std::cell::Cell;

use tokenizer::Token;

use crate::types::InsertionMode;
use crate::{is_parser_whitespace, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn handle_before_head(&'a self, token: &Token) {
        let anything_else = || {
            // Insert an HTML element for a "head" start tag token with no attributes.
            let head_element = self.insert_html_element_for_token(&Token::StartTag {
                name: "head".to_string(),
                self_closing: false,
                self_closing_acknowledged: Cell::new(false),
                attributes: vec![],
            });

            // Set the head element pointer to the newly created head element.
            self.head_element.set(Some(head_element));

            // Switch the insertion mode to "in head".
            self.switch_insertion_mode_to(InsertionMode::InHead);
            // Reprocess the current token.
            self.process_token(token);
        };

        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {}
            Token::Comment { .. } => todo!(),
            Token::Doctype { .. } => todo!(),
            Token::StartTag { name, .. } if name == "html" => todo!(),
            Token::StartTag { name, .. } if name == "head" => todo!(),
            Token::EndTag { name, .. }
                if name == "head" || name == "body" || name == "html" || name == "br" =>
            {
                anything_else()
            }
            Token::EndTag { .. } => todo!(),
            _ => anything_else(),
        }
    }
}
