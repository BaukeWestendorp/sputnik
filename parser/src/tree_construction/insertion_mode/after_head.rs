use tokenizer::Token;

use crate::types::InsertionMode;
use crate::{is_parser_whitespace, log_parser_error, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn handle_after_head(&'a self, token: &Token) {
        macro_rules! anything_else {
            () => {
                // Insert an HTML element for a "body" start tag token with no attributes.
                self.insert_html_element_for_start_tag("body");

                // Switch the insertion mode to "in body".
                self.switch_insertion_mode_to(InsertionMode::InBody);

                // Reprocess the current token.
                self.process_token(token);
            };
        }
        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                // Insert the character.
                self.insert_character(*data);
            }
            Token::Comment { data } => {
                // Insert a comment.
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {
                // Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::StartTag { name, .. } if name == "html" => {
                // Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody, token)
            }
            Token::StartTag { name, .. } if name == "body" => {
                // Insert an HTML element for the token.
                self.insert_html_element_for_token(token);
                // Set the frameset-ok flag to "not ok".
                self.frameset_ok.set(false);
                // Switch the insertion mode to "in body".
                self.switch_insertion_mode_to(InsertionMode::InBody);
            }
            Token::StartTag { name, .. } if name == "frameset" => {
                // Insert an HTML element for the token.
                self.insert_html_element_for_token(token);
                // Switch the insertion mode to "in frameset".
                self.switch_insertion_mode_to(InsertionMode::InFrameset);
            }
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
                // Parse error.
                log_parser_error!();
                // Push the node pointed to by the head element pointer onto the stack of open elements.
                self.open_elements.push(self.head_element.get().unwrap());
                // Process the token using the rules for the "in head" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InHead, token);
                // Remove the node pointed to by the head element pointer from the stack of open elements. (It might not be the current node at this point.)
                self.open_elements
                    .remove_element(self.head_element.get().unwrap());
            }
            Token::EndTag { name, .. } if name == "template" => {
                // Process the token using the rules for the "in head" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InHead, token);
            }
            Token::EndTag { name, .. } if name == "body" || name == "html" || name == "br" => {
                // Act as described in the "anything else" entry below.
                anything_else!();
            }
            Token::StartTag { name, .. } if name == "head" => {
                // Parse error. Ignore the token.
                log_parser_error!();
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
