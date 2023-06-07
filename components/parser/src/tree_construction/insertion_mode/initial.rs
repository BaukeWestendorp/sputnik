use crate::{is_parser_whitespace, log_parser_error, InsertionMode, Parser};
use dom::node::{Node, NodeType};
use tokenizer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn handle_initial(&'a self, token: &Token) {
        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                // Ignore the token.
            }
            Token::Comment { data } => {
                // Insert a comment as the last child of the Document object.
                self.insert_comment_as_last_child_of(data, &self.document);
            }
            Token::Doctype {
                name,
                public_identifier,
                system_identifier,
                ..
            } => {
                // If the DOCTYPE token's name is not "html", or the token's public identifier is not missing, or the token's system identifier is neither missing nor "about:legacy-compat", then there is a parse error.
                if name != &Some("html".to_string())
                    || public_identifier.is_some()
                    || (system_identifier
                        .to_owned()
                        .is_some_and(|id| id == "about:legacy-compat"))
                {
                    log_parser_error!("Bad DOCTYPE");
                }

                // Append a DocumentType node to the Document node, with its name set to the name given in the DOCTYPE token, or the empty string if the name was missing; its public ID set to the public identifier given in the DOCTYPE token, or the empty string if the public identifier was missing; and its system ID set to the system identifier given in the DOCTYPE token, or the empty string if the system identifier was missing.
                let doctype_node = self.allocate_node(Node::new(
                    Some(&self.document),
                    NodeType::DocumentType {
                        name: name.clone().unwrap_or("".to_string()),
                        public_identifier: public_identifier.clone().unwrap_or("".to_string()),
                        system_identifier: system_identifier.clone().unwrap_or("".to_string()),
                    },
                ));
                Node::append(doctype_node, &self.document, false);

                // FIXME: Then, if the document is not an iframe srcdoc document, and the parser cannot change the mode flag is false, and the DOCTYPE token matches one of the conditions in the following list, then set the Document to quirks mode:
                // FIXME: Otherwise, if the document is not an iframe srcdoc document, and the parser cannot change the mode flag is false, and the DOCTYPE token matches one of the conditions in the following list, then then set the Document to limited-quirks mode:

                // Then, switch the insertion mode to "before html".
                self.switch_insertion_mode_to(InsertionMode::BeforeHtml);
            }
            _ => {
                todo!()
            }
        }
    }
}
