use tokenizer::Token;

use crate::types::InsertionMode;
use crate::{is_parser_whitespace, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn handle_in_head(&'a self, token: &Token) {
        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                todo!()
            }
            Token::Comment { .. } => todo!(),
            Token::Doctype { .. } => todo!(),
            Token::StartTag { name, .. } if name == "html" => todo!(),
            Token::StartTag { name, .. }
                if name == "base" || name == "basefont" || name == "bgsound" || name == "link" =>
            {
                todo!()
            }
            Token::StartTag { name, .. } if name == "meta" => todo!(),
            Token::StartTag { name, .. } if name == "title" => todo!(),
            Token::StartTag { name, .. } if name == "noscript" && self.scripting => todo!(),
            Token::StartTag { name, .. } if name == "noframes" || name == "style" => todo!(),
            Token::StartTag { name, .. } if name == "noscript" && !self.scripting => todo!(),
            Token::StartTag { name, .. } if name == "script" => todo!(),
            Token::EndTag { name, .. } if name == "head" => todo!(),
            Token::EndTag { name, .. } if name == "body" || name == "html" || name == "br" => {
                todo!()
            }
            Token::StartTag { name, .. } if name == "template" => todo!(),
            Token::EndTag { name, .. } if name == "template" => todo!(),
            Token::StartTag { name, .. } if name == "head" => todo!(),
            Token::EndTag { .. } => todo!(),
            _ => {
                // Pop the current node (which will be the head element) off the stack of open elements.
                self.open_elements.pop();

                // Switch the insertion mode to "after head".
                self.switch_insertion_mode_to(InsertionMode::AfterHead);

                // Reprocess the token.
                self.process_token(token);
            }
        }
    }
}
