use tokenizer::Token;

use crate::types::InsertionMode;
use crate::{is_parser_whitespace, log_parser_error, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn handle_after_after_body(&'a self, token: &Token) {
        match token {
            Token::Comment { data } => {
                // Insert a comment.
                self.insert_comment_as_last_child_of(data, &self.document);
            }
            Token::Doctype { .. } => {
                // Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody, token);
            }
            Token::Character { data } if is_parser_whitespace(*data) => {
                // Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody, token);
            }
            Token::StartTag { name, .. } if name == "html" => {
                // Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody, token);
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
