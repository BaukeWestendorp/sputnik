use crate::html::tokenizer::Token;

use crate::html::parser::{is_parser_whitespace, log_parser_error, InsertionMode, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn handle_before_head(&'a self, token: &Token) {
        macro_rules! anything_else {
            () => {
                // Insert an HTML element for a "head" start tag token with no attributes.
                let head_element = self.insert_html_element_for_start_tag("head");

                // Set the head element pointer to the newly created head element.
                self.head_element.set(Some(head_element));

                // Switch the insertion mode to "in head".
                self.switch_insertion_mode_to(InsertionMode::InHead);
                // Reprocess the current token.
                self.process_token(token);
            };
        }

        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                // Ignore the token.
            }
            Token::Comment { data } => {
                // Insert a comment.
                self.insert_comment(data)
            }
            Token::Doctype { .. } => {
                // Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::StartTag { name, .. } if name == "html" => {
                // Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody, token);
            }
            Token::StartTag { name, .. } if name == "head" => {
                // Insert an HTML element for the token.
                let head_element = self.insert_html_element_for_token(token);
                // Set the head element pointer to the newly created head element.
                self.head_element.set(Some(head_element));
                // Switch the insertion mode to "in head".
                self.switch_insertion_mode_to(InsertionMode::InHead);
            }
            Token::EndTag { name, .. }
                if name == "head" || name == "body" || name == "html" || name == "br" =>
            {
                anything_else!();
            }
            Token::EndTag { .. } => {
                // Parse error. Ignore the token.
                log_parser_error!();
            }
            _ => {
                anything_else!();
            }
        }
    }
}
