use tokenizer::Token;

use crate::{log_parser_error, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn handle_text(&'a self, token: &Token) {
        match token {
            Token::Character { data } => {
                // Insert the token's character.
                self.insert_character(*data);
            }
            Token::EndOfFile => {
                // Parse error.
                log_parser_error!("Unexpected EOF token in text");

                // FIXME: If the current node is a script element, then set its already started to true.

                // Pop the current node off the stack of open elements.
                self.open_elements.pop();

                // Switch the insertion mode to the original insertion mode and reprocess the token.
                self.switch_insertion_mode_to(self.original_insertion_mode.get().unwrap());
                self.process_token(token);
            }
            Token::EndTag { name, .. } if name == "script" => todo!(),
            _ => {
                // Pop the current node off the stack of open elements.
                self.open_elements.pop();

                // Switch the insertion mode to the original insertion mode.
                self.switch_insertion_mode_to(self.original_insertion_mode.get().unwrap());
            }
        }
    }
}
