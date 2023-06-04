use tokenizer::Token;

use crate::namespace::Namespace;
use crate::types::InsertionMode;
use crate::{is_parser_whitespace, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn handle_before_html(&'a self, token: &Token) {
        match token {
            Token::Doctype { .. } => todo!(),
            Token::Comment { .. } => todo!(),
            Token::Character { data } if is_parser_whitespace(*data) => todo!(),
            Token::StartTag { name, .. } if name == "html" => {
                let element = self.create_element_for_token(token, Namespace::Html, &self.document);
                self.document.append_child(element);
                self.open_elements.push(element);
                self.switch_insertion_mode_to(InsertionMode::BeforeHead);
            }
            Token::EndTag { name, .. }
                if name == "head" || name == "body" || name == "html" || name == "br" =>
            {
                todo!()
            }
            Token::EndTag { .. } => todo!(),
            _ => todo!(),
        }
    }
}
