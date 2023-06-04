use tokenizer::Token;

use crate::dom::Node;
use crate::namespace::Namespace;
use crate::types::InsertionMode;
use crate::{is_parser_whitespace, log_parser_error, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn handle_before_html(&'a self, token: &Token) {
        macro_rules! anything_else {
            () => {
                // Create an html element whose node document is the Document object.
                let html_element = self.create_element(
                    &self.document,
                    &"html".to_string(),
                    Namespace::Html,
                    None,
                    None,
                    false,
                );
                // Append it to the Document object.
                Node::append(html_element, &self.document, false);
                // Put this element in the stack of open elements.
                self.open_elements.push(html_element);

                // Switch the insertion mode to "before head", then reprocess the token.
                self.switch_insertion_mode_to(InsertionMode::BeforeHead);
            };
        }
        match token {
            Token::Doctype { .. } => {
                // Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::Comment { data } => {
                // Insert a comment as the last child of the Document object.
                self.insert_comment_as_last_child_of(data, &self.document)
            }
            Token::Character { data } if is_parser_whitespace(*data) => {
                // Ignore the token.
            }
            Token::StartTag { name, .. } if name == "html" => {
                // Create an element for the token in the HTML namespace, with the Document as the intended parent.
                let element = self.create_element_for_token(token, Namespace::Html, &self.document);
                // Append it to the Document object.
                self.document.append_child(element);
                // Put this element in the stack of open elements.
                self.open_elements.push(element);

                // Switch the insertion mode to "before head".
                self.switch_insertion_mode_to(InsertionMode::BeforeHead);
            }
            Token::EndTag { name, .. }
                if name == "head" || name == "body" || name == "html" || name == "br" =>
            {
                // Act as described in the "anything else" entry below.
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
