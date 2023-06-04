use std::cell::Cell;

use tokenizer::Token;

use crate::types::InsertionMode;
use crate::{is_parser_whitespace, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn handle_after_head(&'a self, token: &Token) {
        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                todo!()
            }
            Token::Comment { .. } => todo!(),
            Token::Doctype { .. } => todo!(),
            Token::StartTag { name, .. } if name == "html" => todo!(),
            Token::StartTag { name, .. } if name == "body" => todo!(),
            Token::StartTag { name, .. } if name == "frameset" => todo!(),
            Token::StartTag { name, .. }
                if name == "base"
                    || name == "basefont"
                    || name == "bgsound"
                    || name == "link"
                    || name == "meta"
                    || name == "noframes"
                    || name == "script"
                    || name == "style"
                    || name == "template"
                    || name == "title" =>
            {
                todo!()
            }
            Token::EndTag { name, .. } if name == "template" => todo!(),
            Token::EndTag { name, .. } if name == "body" || name == "html" || name == "br" => {
                todo!()
            }
            Token::StartTag { name, .. } if name == "head" => todo!(),
            _ => {
                // Insert an HTML element for a "body" start tag token with no attributes.
                self.insert_html_element_for_token(&Token::StartTag {
                    name: "body".to_string(),
                    self_closing: false,
                    self_closing_acknowledged: Cell::new(false),
                    attributes: vec![],
                });

                // Switch the insertion mode to "in body".
                self.switch_insertion_mode_to(InsertionMode::InBody);

                // Reprocess the current token.
                self.process_token(token);
            }
        }
    }
}
