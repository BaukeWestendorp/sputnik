use tokenizer::Token;

use crate::{is_parser_whitespace, log_parser_error, InsertionMode, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn handle_after_body(&'a self, token: &Token) {
        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                // Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody, token);
            }
            Token::Comment { data } => {
                // Insert a comment.
                self.insert_comment_as_last_child_of(data, self.open_elements.first().unwrap());
            }
            Token::Doctype { .. } => {
                // Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::StartTag { name, .. } if name == "html" => {
                // Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody, token);
            }
            Token::EndTag { name, .. } if name == "html" => {
                // FIXME: If the parser was created as part of the HTML fragment parsing algorithm, this is a parse error; ignore the token. (fragment case)

                // Otherwise, switch the insertion mode to "after after body".
                self.switch_insertion_mode_to(InsertionMode::AfterAfterBody);
            }
            Token::EndOfFile => {
                self.stop_parsing();
            }
            _ => {
                // Parse error.
                log_parser_error!();
                // Switch the insertion mode to "in body" and reprocess the token.
                self.switch_insertion_mode_to(InsertionMode::InBody);
            }
        }
    }
}
