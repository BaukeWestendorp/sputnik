use std::cell::Cell;

use tokenizer::Token;

use crate::dom::{Node, NodeType};
use crate::namespace::Namespace;
use crate::tree_construction::stack_of_open_elements;
use crate::types::InsertionMode;
use crate::{is_parser_whitespace, Parser};

macro_rules! log_parser_error {
    ($message:expr) => {
        eprintln!(
            "\x1b[31m[Parser Error ({}:{})]: {}\x1b[0m",
            file!(),
            line!(),
            $message
        );
    };
    () => {
        eprintln!("\x1b[31m[Parser Error ({}:{})]\x1b[0m", file!(), line!());
    };
}

impl<'a> Parser<'a> {
    fn switch_insertion_mode_to(&self, insertion_mode: InsertionMode) {
        self.insertion_mode.set(insertion_mode)
    }

    pub(crate) fn handle_initial(&'a self, token: &Token) {
        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {}
            Token::Comment { .. } => {
                todo!()
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

                self.switch_insertion_mode_to(InsertionMode::BeforeHtml);
            }
            _ => {
                todo!()
            }
        }
    }

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

    pub(crate) fn handle_before_head(&'a self, token: &Token) {
        let anything_else = || {
            // Insert an HTML element for a "head" start tag token with no attributes.
            let head_element = self.insert_html_element_for_token(&Token::StartTag {
                name: "head".to_string(),
                self_closing: false,
                self_closing_acknowledged: Cell::new(false),
                attributes: vec![],
            });

            // Set the head element pointer to the newly created head element.
            self.head_element.set(Some(head_element));

            // Switch the insertion mode to "in head".
            self.switch_insertion_mode_to(InsertionMode::InHead);
            // Reprocess the current token.
            self.process_token(token);
        };

        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {}
            Token::Comment { .. } => todo!(),
            Token::Doctype { .. } => todo!(),
            Token::StartTag { name, .. } if name == "html" => todo!(),
            Token::StartTag { name, .. } if name == "head" => todo!(),
            Token::EndTag { name, .. }
                if name == "head" || name == "body" || name == "html" || name == "br" =>
            {
                anything_else()
            }
            Token::EndTag { .. } => todo!(),
            _ => anything_else(),
        }
    }

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
                self.process_token(token)
            }
        }
    }

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

    pub(crate) fn handle_in_body(&'a self, token: &Token) {
        const INVALID_EOF_TAGS: &[&str] = &[
            "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc", "tbody", "td",
            "tfoot", "th", "thead", "tr", "body", "html",
        ];

        match token {
            Token::Character { data } if data == &'\u{0000}' => {
                // Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::Character { data } if is_parser_whitespace(*data) => {
                // Reconstruct the active formatting elements, if any.
                self.active_formatting_elements.reconstruct_if_any();

                // Insert the token's character.
                self.insert_character(*data);
            }
            Token::Character { data } => {
                // Reconstruct the active formatting elements, if any.
                self.active_formatting_elements.reconstruct_if_any();
                // Insert the token's character.
                self.insert_character(*data);
                // Set the frameset-ok flag to "not ok".
                self.frameset_ok.set(false);
            }
            Token::Comment { data } => {
                // Insert a comment.
                self.insert_comment(data)
            }
            Token::Doctype { .. } => {
                // Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::StartTag { name, .. } if name == "html" => todo!(),
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
                // Process the token using the rules for the "in head" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InHead, token);
            }
            Token::EndTag { name, .. } if name == "template" => {
                // Process the token using the rules for the "in head" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InHead, token);
            }
            Token::StartTag { name, .. } if name == "body" => todo!(),
            Token::StartTag { name, .. } if name == "frameset" => todo!(),
            Token::EndOfFile => {
                // If the stack of template insertion modes is not empty, then process the token using the rules for the "in template" insertion mode.
                // FIXME: Implement

                // Otherwise, follow these steps:

                // If there is a node in the stack of open elements that is not either a dd element, a dt element, an li element, an optgroup element, an option element, a p element, an rb element, an rp element, an rt element, an rtc element, a tbody element, a td element, a tfoot element, a th element, a thead element, a tr element, the body element, or the html element, then this is a parse error.
                if !self.open_elements.contains_one_of_tags(INVALID_EOF_TAGS) {
                    log_parser_error!();
                };

                // Stop parsing.
                self.stop_parsing();
            }
            Token::EndTag { name, .. } if name == "body" => {
                // If the stack of open elements does not have a body element in scope,
                if !self
                    .open_elements
                    .has_element_with_tag_name_in_scope("body")
                {
                    // this is a parse error; ignore the token.
                    log_parser_error!();
                    return;
                }

                // Otherwise, if there is a node in the stack of open elements that is not either a dd element, a dt element, an li element, an optgroup element, an option element, a p element, an rb element, an rp element, an rt element, an rtc element, a tbody element, a td element, a tfoot element, a th element, a thead element, a tr element, the body element, or the html element, then this is a parse error.
                if !self.open_elements.contains_one_of_tags(INVALID_EOF_TAGS) {
                    log_parser_error!();
                }

                // Switch the insertion mode to "after body".
                self.switch_insertion_mode_to(InsertionMode::AfterBody);
            }
            Token::EndTag { name, .. } if name == "html" => {
                // 1. If the stack of open elements does not have a body element in scope, this is a parse error; ignore the token.
                if !self
                    .open_elements
                    .has_element_with_tag_name_in_scope("body")
                {
                    log_parser_error!();
                    return;
                }
                // 2. Otherwise, if there is a node in the stack of open elements that is not either a dd element, a dt element, an li element, an optgroup element, an option element, a p element, an rb element, an rp element, an rt element, an rtc element, a tbody element, a td element, a tfoot element, a th element, a thead element, a tr element, the body element, or the html element, then this is a parse error.

                // 3. Switch the insertion mode to "after body".
                self.switch_insertion_mode_to(InsertionMode::AfterBody);
                // 4. Reprocess the token.
                self.process_token(token);
            }
            Token::StartTag { name, .. }
                if name == "address"
                    || name == "article"
                    || name == "aside"
                    || name == "blockquote"
                    || name == "center"
                    || name == "details"
                    || name == "dialog"
                    || name == "dir"
                    || name == "div"
                    || name == "dl"
                    || name == "fieldset"
                    || name == "figcaption"
                    || name == "figure"
                    || name == "footer"
                    || name == "header"
                    || name == "hgroup"
                    || name == "main"
                    || name == "menu"
                    || name == "nav"
                    || name == "ol"
                    || name == "p"
                    || name == "search"
                    || name == "section"
                    || name == "summary"
                    || name == "ul" =>
            {
                // If the stack of open elements has a p element in button scope, then close a p element.
                if self
                    .open_elements
                    .has_element_with_tag_name_in_button_scope("p")
                {
                    self.close_a_p_element();
                }
                // Insert an HTML element for the token.
                self.insert_html_element_for_token(token);
            }
            Token::StartTag { name, .. }
                if name == "h1"
                    || name == "h2"
                    || name == "h3"
                    || name == "h4"
                    || name == "h5"
                    || name == "h6" =>
            {
                todo!()
            }
            Token::StartTag { name, .. } if name == "pre" || name == "listing" => todo!(),
            Token::StartTag { name, .. } if name == "form" => {
                // If the form element pointer is not null, and there is no template element on the stack of open elements,
                if self.form_element.get().is_some()
                    && self.open_elements.contains_one_of_tags(&["template"])
                {
                    // then this is a parse error; ignore the token.
                    return;
                }
                // Otherwise:
                // If the stack of open elements has a p element in button scope,
                if self
                    .open_elements
                    .has_element_with_tag_name_in_button_scope("p")
                {
                    // then close a p element.
                    self.close_a_p_element();
                }
                // Insert an HTML element for the token,
                self.insert_html_element_for_token(token);
                // and, if there is no template element on the stack of open elements,
                if self.open_elements.contains_one_of_tags(&["template"]) {
                    // set the form element pointer to point to the element created.
                    self.form_element.set(self.open_elements.current_node());
                }
            }
            Token::StartTag { name, .. } if name == "li" => {
                // 1. Set the frameset-ok flag to "not ok".
                self.frameset_ok.set(false);

                // 2. Initialize node to be the current node (the bottommost node of the stack).
                for node in self.open_elements.elements.borrow().iter().rev() {
                    // 3. Loop: If node is an li element, then run these substeps:
                    if node.is_element_with_tag("li") {
                        // 3.1. Generate implied end tags, except for li elements.
                        self.generate_implied_end_tags_except_for(Some("li"));
                        // 3.2. If the current node is not an li element, then this is a parse error.
                        if !self
                            .open_elements
                            .current_node()
                            .unwrap()
                            .is_element_with_tag("li")
                        {
                            log_parser_error!();
                        }
                        // 3.3. Pop elements from the stack of open elements until an li element has been popped from the stack.
                        self.open_elements
                            .pop_elements_until_element_has_been_popped("li");

                        // 3.4. Jump to the step labeled done below.
                        break;
                    }

                    // 4. If node is in the special category, but is not an address, div, or p element, then jump to the step labeled done below.
                    if node.is_element_with_one_of_tags(stack_of_open_elements::SPECIAL_TAGS)
                        && !node.is_element_with_one_of_tags(&["address", "div", "p"])
                    {
                        break;
                    }

                    // 5. Otherwise, set node to the previous entry in the stack of open elements and return to the step labeled loop.
                }

                // 6. Done: If the stack of open elements has a p element in button scope, then close a p element.
                if self
                    .open_elements
                    .has_element_with_tag_name_in_button_scope("p")
                {
                    self.close_a_p_element();
                }

                // 7. Finally, insert an HTML element for the token.
                self.insert_html_element_for_token(token);
            }
            Token::StartTag { name, .. } if name == "dd" || name == "dt" => todo!(),
            Token::StartTag { name, .. } if name == "plaintext" => todo!(),
            Token::StartTag { name, .. } if name == "button" => todo!(),
            Token::EndTag { name, .. }
                if name == "address"
                    || name == "article"
                    || name == "aside"
                    || name == "blockquote"
                    || name == "button"
                    || name == "center"
                    || name == "details"
                    || name == "dialog"
                    || name == "dir"
                    || name == "div"
                    || name == "dl"
                    || name == "fieldset"
                    || name == "figcaption"
                    || name == "figure"
                    || name == "footer"
                    || name == "header"
                    || name == "hgroup"
                    || name == "listing"
                    || name == "main"
                    || name == "menu"
                    || name == "nav"
                    || name == "ol"
                    || name == "pre"
                    || name == "search"
                    || name == "section"
                    || name == "summary"
                    || name == "ul" =>
            {
                // If the stack of open elements does not have an element in scope that is an HTML element with the same tag name as that of the token,
                if self.open_elements.has_element_with_tag_name_in_scope(name) {
                    // then this is a parse error; ignore the token.
                    log_parser_error!();
                    return;
                }
                // Otherwise, run these steps:
                // 1. Generate implied end tags.
                self.generate_implied_end_tags_except_for(None);
                // 2. If the current node is not an HTML element with the same tag name as that of the token,
                if !self
                    .open_elements
                    .current_node()
                    .unwrap()
                    .is_element_with_tag(token.tag_name().unwrap().as_str())
                {
                    // then this is a parse error.
                    log_parser_error!("Found closing tag, but current node is not an HTML element with the same tag name.");
                }
                // 3. Pop elements from the stack of open elements until an HTML element with the same tag name as the token has been popped from the stack.
                self.open_elements
                    .pop_elements_until_element_has_been_popped(name);
            }
            Token::EndTag { name, .. } if name == "form" => {
                // If there is no template element on the stack of open elements, then run these substeps:
                if !self.open_elements.contains_one_of_tags(&["template"]) {
                    // 1. Let node be the element that the form element pointer is set to, or null if it is not set to an element.
                    let node = self.form_element.get();
                    // 2. Set the form element pointer to null.
                    self.form_element.set(None);
                    // 3. If node is null or if the stack of open elements does not have node in scope,
                    if node.is_none() || self.open_elements.has_element_in_scope(node.unwrap()) {
                        // then this is a parse error;
                        log_parser_error!();
                        // return and ignore the token.
                        return;
                    }
                    // 4. Generate implied end tags.
                    self.generate_implied_end_tags_except_for(None);
                    // 5. If the current node is not node,
                    if self.open_elements.current_node().unwrap() == node.unwrap() {
                        // then this is a parse error.
                        log_parser_error!();
                    }
                    // 6. Remove node from the stack of open elements.
                    self.open_elements.remove_element(node.unwrap());
                } else {
                    // 1. If the stack of open elements does not have a form element in scope, then this is a parse error; return and ignore the token.
                    // 2. Generate implied end tags.
                    // 3. If the current node is not a form element, then this is a parse error.
                    // 4. Pop elements from the stack of open elements until a form element has been popped from the stack.
                    todo!();
                }
            }
            Token::EndTag { name, .. } if name == "p" => {
                // If the stack of open elements does not have a p element in button scope, then this is a parse error; insert an HTML element for a "p" start tag token with no attributes.
                if !self
                    .open_elements
                    .has_element_with_tag_name_in_button_scope("p")
                {
                    log_parser_error!("Found </p> closing tag in invalid scope.");
                    self.insert_html_element_for_token(&Token::StartTag {
                        name: "p".to_string(),
                        self_closing: false,
                        self_closing_acknowledged: Cell::new(false),
                        attributes: vec![],
                    });
                }

                // Close a p element.
                self.close_a_p_element();
            }
            Token::EndTag { name, .. } if name == "li" => {
                // If the stack of open elements does not have an li element in list item scope, then this is a parse error; ignore the token.
                if self
                    .open_elements
                    .has_element_with_tag_name_in_list_item_scope("li")
                {
                    log_parser_error!();
                    return;
                }

                // Otherwise, run these steps:
                // 1. Generate implied end tags, except for li elements.
                self.generate_implied_end_tags_except_for(Some("li"));

                // 2. If the current node is not an li element, then this is a parse error.
                if !self
                    .open_elements
                    .current_node()
                    .unwrap()
                    .is_element_with_tag("li")
                {
                    log_parser_error!();
                }

                // 3. Pop elements from the stack of open elements until an li element has been popped from the stack.
                self.open_elements
                    .pop_elements_until_element_has_been_popped("li");
            }
            Token::EndTag { name, .. } if name == "dd" || name == "dt" => {
                todo!()
            }
            Token::EndTag { name, .. }
                if name == "h1"
                    || name == "h2"
                    || name == "h3"
                    || name == "h4"
                    || name == "h5"
                    || name == "h6" =>
            {
                todo!()
            }
            Token::StartTag { name, .. } if name == "a" => {
                use super::list_of_active_formatting_elements::Position;

                // If the list of active formatting elements contains an a element between the end of the list and the last marker on the list (or the start of the list if there is no marker on the list),
                if self.active_formatting_elements.contains_element_between(
                    Position::End,
                    Position::LastMarkerOrElseStart,
                    "a",
                ) {
                    // then this is a parse error;
                    log_parser_error!();

                    // run the adoption agency algorithm for the token,
                    self.run_adoption_agency_algorithm_for_token(token);

                    // then remove that element from the list of active formatting elements and the stack of open elements if the adoption agency algorithm didn't already remove it (it might not have if the element is not in table scope).
                    todo!()
                }

                // Reconstruct the active formatting elements, if any.
                self.active_formatting_elements.reconstruct_if_any();
                // Insert an HTML element for the token.
                let element = self.insert_html_element_for_token(token);
                // Push onto the list of active formatting elements that element.
                self.active_formatting_elements.push_element(element);
            }
            Token::StartTag { name, .. }
                if name == "b"
                    || name == "big"
                    || name == "code"
                    || name == "em"
                    || name == "font"
                    || name == "i"
                    || name == "s"
                    || name == "small"
                    || name == "strike"
                    || name == "strong"
                    || name == "tt"
                    || name == "u" =>
            {
                // Reconstruct the active formatting elements, if any.
                self.active_formatting_elements.reconstruct_if_any();
                // Insert an HTML element for the token.
                let element = self.insert_html_element_for_token(token);
                // Push onto the list of active formatting elements that element.
                self.active_formatting_elements.push_element(element);
            }
            Token::StartTag { name, .. } if name == "nobr" => todo!(),
            Token::EndTag { name, .. }
                if name == "a"
                    || name == "b"
                    || name == "big"
                    || name == "code"
                    || name == "em"
                    || name == "font"
                    || name == "i"
                    || name == "nobr"
                    || name == "s"
                    || name == "small"
                    || name == "strike"
                    || name == "strong"
                    || name == "tt"
                    || name == "u" =>
            {
                // Run the adoption agency algorithm for the token.
                self.run_adoption_agency_algorithm_for_token(token);
            }

            Token::StartTag { name, .. }
                if name == "applet" || name == "marquee" || name == "object" =>
            {
                todo!()
            }
            Token::EndTag { name, .. }
                if name == "applet" || name == "marquee" || name == "object" =>
            {
                todo!()
            }
            Token::StartTag { name, .. } if name == "table" => {
                // If the Document is not set to quirks mode, and the stack of open elements has a p element in button scope, then close a p element.
                // FIXME: Implement
                // Insert an HTML element for the token.
                self.insert_html_element_for_token(token);
                // Set the frameset-ok flag to "not ok".
                self.frameset_ok.set(false);
                // Switch the insertion mode to "in table".
                self.switch_insertion_mode_to(InsertionMode::InTable);
            }
            Token::EndTag { name, .. } if name == "br" => todo!(),
            Token::StartTag { name, .. }
                if name == "area"
                    || name == "br"
                    || name == "embed"
                    || name == "img"
                    || name == "keygen"
                    || name == "wbr" =>
            {
                // Reconstruct the active formatting elements, if any.
                self.active_formatting_elements.reconstruct_if_any();
                // Insert an HTML element for the token.
                self.insert_html_element_for_token(token);
                // Immediately pop the current node off the stack of open elements.
                self.open_elements.pop();
                // Acknowledge the token's self-closing flag, if it is set.
                token.acknowledge_self_closing_flag_if_set();
                // Set the frameset-ok flag to "not ok".
                self.frameset_ok.set(false);
            }
            Token::StartTag { name, .. } if name == "input" => {
                // Reconstruct the active formatting elements, if any.
                self.active_formatting_elements.reconstruct_if_any();

                // Insert an HTML element for the token.
                self.insert_html_element_for_token(token);

                // Immediately pop the current node off the stack of open elements.
                self.open_elements.pop();

                // Acknowledge the token's self-closing flag, if it is set.
                token.acknowledge_self_closing_flag_if_set();

                // If the token does not have an attribute with the name "type", or if it does, but that attribute's value is not an ASCII case-insensitive match for the string "hidden",
                if let Token::StartTag { attributes, .. } = token {
                    let type_attr = attributes.iter().find(|attr| attr.name == "type");
                    if type_attr.is_none()
                        || type_attr.unwrap().value.eq_ignore_ascii_case("hidden")
                    {
                        // then: set the frameset-ok flag to "not ok".
                        self.frameset_ok.set(false);
                    }
                }
            }
            Token::StartTag { name, .. }
                if name == "param" || name == "source" || name == "track" =>
            {
                todo!()
            }
            Token::StartTag { name, .. } if name == "hr" => todo!(),
            Token::StartTag { name, .. } if name == "image" => todo!(),
            Token::StartTag { name, .. } if name == "textarea" => todo!(),
            Token::StartTag { name, .. } if name == "xmp" => todo!(),
            Token::StartTag { name, .. } if name == "iframe" => todo!(),
            Token::StartTag { name, .. } if name == "noembed" => todo!(),
            Token::StartTag { name, .. } if name == "noscript" && self.scripting => {
                todo!()
            }
            Token::StartTag { name, .. } if name == "select" => todo!(),
            Token::StartTag { name, .. } if name == "optgroup" || name == "option" => todo!(),
            Token::StartTag { name, .. } if name == "rb" || name == "rtc" => todo!(),
            Token::StartTag { name, .. } if name == "rp" || name == "rt" => todo!(),
            Token::StartTag { name, .. } if name == "math" => todo!(),
            Token::StartTag { name, .. } if name == "svg" => todo!(),
            Token::StartTag { name, .. }
                if name == "caption"
                    || name == "col"
                    || name == "colgroup"
                    || name == "frame"
                    || name == "head"
                    || name == "tbody"
                    || name == "td"
                    || name == "tfoot"
                    || name == "th"
                    || name == "thead"
                    || name == "tr" =>
            {
                // Parser error. Ignore the token.
                log_parser_error!();
            }
            Token::StartTag { .. } => {
                // Reconstruct the active formatting elements, if any.
                self.active_formatting_elements.reconstruct_if_any();
                // Insert an HTML element for the token.
                self.insert_html_element_for_token(token);
            }
            Token::EndTag { .. } => self.in_body_any_other_end_tag(token),
        }
    }

    fn in_body_any_other_end_tag(&self, token: &Token) {
        // 1. Initialize node to be the current node (the bottommost node of the stack).
        for node in self.open_elements.elements.borrow().iter().rev() {
            // 2. Loop: If node is an HTML element with the same tag name as the token, then:
            let token_tag_name = token.tag_name().expect("token should be EndTag");
            if node.is_element_with_tag(&token_tag_name) {
                // 2.1. Generate implied end tags, except for HTML elements with the same tag name as the token.
                self.generate_implied_end_tags_except_for(Some(&token_tag_name));
                // 2.2. If node is not the current node, then this is a parse error.
                if *node == self.open_elements.current_node().unwrap() {
                    log_parser_error!();
                }
                // 2.3. Pop all the nodes from the current node up to node, including node,
                while *node == self.open_elements.current_node().unwrap() {
                    self.open_elements.pop();
                }
                // then stop these steps.
                break;
            } else {
                // 3. Otherwise, if node is in the special category,
                if node.is_element_with_one_of_tags(stack_of_open_elements::SPECIAL_TAGS) {
                    // then this is a parse error; ignore the token,
                    log_parser_error!();
                    // and return.
                    return;
                }

                // 4. Set node to the previous entry in the stack of open elements.
                // 5 Return to the step labeled loop.
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#close-a-p-element
    fn close_a_p_element(&self) {
        // Generate implied end tags, except for p elements.
        self.generate_implied_end_tags_except_for(Some("p"));

        // If the current node is not a p element, then this is a parse error.
        if !self
            .open_elements
            .current_node()
            .unwrap()
            .is_element_with_tag("p")
        {
            log_parser_error!();
        }

        // Pop elements from the stack of open elements until a p element has been popped from the stack.
        self.open_elements
            .pop_elements_until_element_has_been_popped("p");
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#adoption-agency-algorithm
    fn run_adoption_agency_algorithm_for_token(&'a self, token: &Token) {
        // 1. Let subject be token's tag name.
        let subject = token.tag_name().expect("token should be EndTag");

        // 2. If the current node is an HTML element whose tag name is subject, and the current node is not in the list of active formatting elements, then pop the current node off the stack of open elements and return.
        if self
            .open_elements
            .current_node()
            .unwrap()
            .is_element_with_tag(&subject)
            && !self
                .active_formatting_elements
                .contains(self.open_elements.current_node().unwrap())
        {
            self.open_elements.pop();
            return;
        }

        // 3. Let outer loop counter be 0.
        let mut outer_loop_counter = 0;

        // 4. While true:
        loop {
            // 4.1 If outer loop counter is greater than or equal to 8, then return.
            if outer_loop_counter >= 8 {
                return;
            }

            // 4.2 Increment outer loop counter by 1.
            outer_loop_counter += 1;
            // 4.3 Let formatting element be the last element in the list of active formatting elements that:
            //     * is between the end of the list and the last marker in the list, if any, or the start of the list otherwise, and
            //     * has the tag name subject.
            let formatting_element = self
                .active_formatting_elements
                .last_element_with_tag_name_before_marker(&subject);

            // If there is no such element, then return and instead act as described in the "any other end tag" entry above.
            if formatting_element.is_none() {
                self.in_body_any_other_end_tag(token);
                return;
            }
            let formatting_element = formatting_element.unwrap();

            // 4.4 If formatting element is not in the stack of open elements,
            if !self.open_elements.contains(formatting_element) {
                // then this is a parse error;
                log_parser_error!();
                // remove the element from the list
                self.active_formatting_elements.remove(formatting_element);
                // and return.
                return;
            }

            // 4.5 If formatting element is in the stack of open elements, but the element is not in scope,
            if !self.open_elements.has_element_in_scope(formatting_element) {
                // then this is a parse error; return.
                log_parser_error!();
                return;
            }

            // 4.6 If formatting element is not the current node,
            if formatting_element != self.open_elements.current_node().unwrap() {
                // this is a parse error. (But do not return.)
                log_parser_error!();
            }

            // 4.7 Let furthest block be the topmost node in the stack of open elements that is lower in the stack than formatting element, and is an element in the special category. There might not be one.
            let furthest_block = self
                .open_elements
                .topmost_special_node_below(formatting_element);

            // 4.8 If there is no furthest block, then the UA must first pop all the nodes from the bottom of the stack of open elements, from the current node up to and including formatting element,
            if furthest_block.is_none() {
                while formatting_element != self.open_elements.current_node().unwrap() {
                    self.open_elements.pop();
                }
                self.open_elements.pop();

                // then remove formatting element from the list of active formatting elements,
                self.active_formatting_elements.remove(formatting_element);
                // and finally return.
                return;
            }
            let furthest_block = furthest_block.unwrap();

            // 4.9 Let common ancestor be the element immediately above formatting element in the stack of open elements.
            let common_ancestor = self
                .open_elements
                .element_immediately_above(formatting_element);

            // 4.10 Let a bookmark note the position of formatting element in the list of active formatting elements relative to the elements on either side of it in the list.
            let mut bookmark = self
                .active_formatting_elements
                .first_index_of(formatting_element)
                .unwrap();

            // 4.11 Let node and last node be furthest block.
            let mut node = furthest_block;
            let mut last_node = furthest_block;

            let node_above_node = self.open_elements.element_immediately_above(node);

            // 4.12 Let inner loop counter be 0.
            let mut inner_loop_count = 0;

            // 4.13 While true:
            loop {
                // 4.13.1 Increment inner loop counter by 1.
                inner_loop_count += 1;

                // 4.13.2 Let node be the element immediately above node in the stack of open elements, or if node is no longer in the stack of open elements (e.g. because it got removed by this algorithm), the element that was immediately above node in the stack of open elements before node was removed.
                if let Some(node_above_node) = node_above_node {
                    node = node_above_node;
                }

                // 4.13.3 If node is formatting element, then break.
                if node == formatting_element {
                    break;
                }

                // 4.13.4 If inner loop counter is greater than 3 and node is in the list of active formatting elements, then remove node from the list of active formatting elements.
                if inner_loop_count > 3 && self.active_formatting_elements.contains(node) {
                    self.active_formatting_elements.remove(node);
                }

                // 4.13.5 If node is not in the list of active formatting elements, then remove node from the stack of open elements and continue.
                if !self.active_formatting_elements.contains(node) {
                    self.open_elements.remove_element(node);
                    continue;
                }

                // 4.13.6 Create an element for the token for which the element node was created, in the HTML namespace, with common ancestor as the intended parent;
                let new_element =
                    self.create_element_for_token(token, Namespace::Html, common_ancestor.unwrap());

                // replace the entry for node in the list of active
                //       formatting elements with an entry for the new element,
                self.active_formatting_elements.replace(node, new_element);

                // replace the entry for node in the stack of open elements
                //       with an entry for the new element,
                self.open_elements.replace(node, new_element);

                // and let node be the new element.
                node = new_element;

                // 4.13.7 If last node is furthest block, then move the aforementioned bookmark to be immediately after the new node in the list of active formatting elements.
                if last_node == furthest_block {
                    bookmark = self
                        .active_formatting_elements
                        .first_index_of(node)
                        .unwrap()
                        + 1
                }

                // 4.13.8 Append last node to node.
                node.append_child(last_node);

                // 4.13.9 Set last node to node.
                last_node = node;
            }

            // 14. Insert whatever last node ended up being in the previous step at the appropriate place for inserting a node, but using common ancestor as the override target.
            let adjusted_insertion_location =
                self.appropriate_place_for_inserting_node(common_ancestor);
            adjusted_insertion_location.insert(last_node);

            // 15. Create an element for the token for which formatting element was created, in the HTML namespace, with furthest block as the intended parent.
            let new_element = self.create_element_for_token(token, Namespace::Html, furthest_block);

            // 16. Take all of the child nodes of furthest block and append them to the element created in the last step.
            for child in furthest_block.child_nodes().iter() {
                new_element.append_child(child);
            }

            // 17. Append that new element to furthest block.
            furthest_block.append_child(new_element);

            // 18. Remove formatting element from the list of active formatting elements,
            self.active_formatting_elements.remove(formatting_element);
            // and insert the new element into the list of active formatting elements at the position of the aforementioned bookmark.
            self.active_formatting_elements
                .insert(bookmark, new_element);

            // 19. Remove formatting element from the stack of open elements,
            self.open_elements.remove_element(formatting_element);

            // and insert the new element into the stack of open elements immediately below the position of furthest block in that stack.
            self.open_elements
                .insert_immediately_below(new_element, furthest_block);
        }
    }
}
