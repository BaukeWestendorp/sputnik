use dom::QualifiedName;
use tokenizer::{Token, Tokenizer};
use tree_builder::TreeBuilder;

pub mod tree_builder;

#[allow(unused)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

#[allow(unused)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
enum GenericParsingAlgorithm {
    RawText,
    RCData,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
enum FramesetState {
    Ok,
    NotOk,
}

#[derive(PartialEq, Eq, PartialOrd, Debug, Clone)]
pub struct Parser<T: TreeBuilder> {
    tree_builder: T,
    insertion_mode: InsertionMode,
    original_insertion_mode: InsertionMode,
    referenced_insertion_mode: Option<InsertionMode>,
    tokenizer: Tokenizer,
    reprocess_current_token: bool,
    open_elements: Vec<T::Handle>,
    // foster_parenting: bool,
    // scripting_flag: bool,
    frameset_ok: FramesetState,
}

const fn is_parser_whitespace(string: char) -> bool {
    if let '\t' | '\u{000a}' | '\u{000c}' | '\u{000d}' | '\u{0020}' = string {
        return true;
    }
    false
}

impl<T: TreeBuilder> Parser<T> {
    pub fn new(tree_builder: T, input: &str) -> Self {
        Self {
            tree_builder,
            insertion_mode: InsertionMode::Initial,
            original_insertion_mode: InsertionMode::Initial,
            referenced_insertion_mode: None,
            tokenizer: Tokenizer::new(input),
            reprocess_current_token: false,
            open_elements: Vec::new(),
            // foster_parenting: false,
            // scripting_flag: false,
            frameset_ok: FramesetState::Ok,
        }
    }

    fn push_element_to_stack_of_open_elements(&mut self, element: T::Handle) {
        self.open_elements.push(element);
    }

    fn pop_current_element_off_stack_of_open_elements(&mut self) {
        self.open_elements.pop();
    }

    fn remove_element_from_stack_of_open_elements(&mut self, element: T::Handle) {
        if let Some(index) = self
            .open_elements
            .iter()
            .position(|e| self.tree_builder.is_same_as(e, &element))
        {
            self.open_elements.remove(index);
        }
    }

    fn switch_insertion_mode_to(&mut self, insertion_mode: InsertionMode) {
        self.insertion_mode = insertion_mode
    }

    fn reprocess_token(&mut self) {
        self.reprocess_current_token = true;
    }

    fn process_token(&mut self, token: &Token) {
        let mode = match self.referenced_insertion_mode {
            Some(insertion_mode) => insertion_mode,
            None => self.insertion_mode,
        };

        match mode {
            InsertionMode::Initial => self.handle_initial_insertion_mode(token),
            InsertionMode::BeforeHtml => self.handle_before_html_insertion_mode(token),
            InsertionMode::BeforeHead => self.handle_before_head_insertion_mode(token),
            InsertionMode::InHead => self.handle_in_head_insertion_mode(token),
            InsertionMode::InHeadNoscript => todo!("InsertionMode::InHeadNoscript"),
            InsertionMode::AfterHead => self.handle_after_head_insertion_mode(token),
            InsertionMode::InBody => todo!("InsertionMode::InBody"),
            InsertionMode::Text => self.handle_text_insertion_mode(token),
            InsertionMode::InTable => todo!("InsertionMode::InTable"),
            InsertionMode::InTableText => todo!("InsertionMode::InTableText"),
            InsertionMode::InCaption => todo!("InsertionMode::InCaption"),
            InsertionMode::InColumnGroup => todo!("InsertionMode::InColumnGroup"),
            InsertionMode::InTableBody => todo!("InsertionMode::InTableBody"),
            InsertionMode::InRow => todo!("InsertionMode::InRow"),
            InsertionMode::InCell => todo!("InsertionMode::InCell"),
            InsertionMode::InSelect => todo!("InsertionMode::InSelect"),
            InsertionMode::InSelectInTable => todo!("InsertionMode::InSelectInTable"),
            InsertionMode::InTemplate => todo!("InsertionMode::InTemplate"),
            InsertionMode::AfterBody => todo!("InsertionMode::AfterBody"),
            InsertionMode::InFrameset => todo!("InsertionMode::InFrameset"),
            InsertionMode::AfterFrameset => todo!("InsertionMode::AfterFrameset"),
            InsertionMode::AfterAfterBody => todo!("InsertionMode::AfterAfterBody"),
            InsertionMode::AfterAfterFrameset => todo!("InsertionMode::AfterAfterFrameset"),
        }
    }

    fn process_token_using_rules_of(&mut self, insertion_mode: InsertionMode) {
        self.referenced_insertion_mode = Some(insertion_mode);
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode
    fn handle_initial_insertion_mode(&mut self, token: &Token) {
        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                // SPEC: Ignore the token.
            }
            Token::Comment { data } => {
                // SPEC: Insert a comment as the last child of the Document object.
                self.tree_builder.insert_comment(&data);
            }
            Token::Doctype {
                name,
                public_identifier,
                system_identifier,
                ..
            } => {
                // SPEC: If the DOCTYPE token's name is not "html",
                //       or the token's public identifier is not missing,
                //       or the token's system identifier is neither missing nor "about:legacy-compat",
                if name != &Some("html".to_string())
                    || public_identifier != &None
                    || (system_identifier != &None
                        && system_identifier != &Some("about:legacy-compat".to_string()))
                {
                    // SPEC: then there is a parse error.
                    self.tree_builder.parser_error(None)
                }

                // SPEC: Append a DocumentType node to the Document node,
                //       with its name set to the name given in the DOCTYPE token, or the empty string if the name was missing;
                //       its public ID set to the public identifier given in the DOCTYPE token, or the empty string if the public identifier was missing;
                //       and its system ID set to the system identifier given in the DOCTYPE token, or the empty string if the system identifier was missing.
                self.tree_builder.append_doctype_to_document(
                    name.clone().unwrap_or_default().as_str(),
                    public_identifier.clone().unwrap_or_default().as_str(),
                    system_identifier.clone().unwrap_or_default().as_str(),
                );

                // SPEC: Then, if the document is not an iframe srcdoc document,
                //       and the parser cannot change the mode flag is false,
                //       and the DOCTYPE token matches one of the conditions in the following list,
                //       then set the Document to quirks mode:
                // FIXME: Implement

                // SPEC: Otherwise, if the document is not an iframe srcdoc document,
                //       and the parser cannot change the mode flag is false,
                //       and the DOCTYPE token matches one of the conditions in the following list,
                //       then then set the Document to limited-quirks mode:
                // FIXME: Implement

                // SPEC: The system identifier and public identifier strings must be compared to
                //       the values given in the lists above in an ASCII case-insensitive manner.
                //       A system identifier whose value is the empty string
                //       is not considered missing for the purposes of the conditions above.
                // FIXME: Implement

                // SPEC: Then, switch the insertion mode to "before html".
                self.switch_insertion_mode_to(InsertionMode::BeforeHtml);
            }
            _ => {
                // SPEC: If the document is not an iframe srcdoc document, then this is a parse error;
                //       if the parser cannot change the mode flag is false, set the Document to quirks mode.
                // FIXME: Implement

                // SPEC: In any case, switch the insertion mode to "before html",
                //       then reprocess the token.
                self.switch_insertion_mode_to(InsertionMode::BeforeHtml);
                self.reprocess_token();
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-before-html-insertion-mode
    fn handle_before_html_insertion_mode(&mut self, token: &Token) {
        match token {
            Token::Doctype { .. } => {
                // SPEC: Parse error. Ignore the token.
                self.tree_builder.parser_error(None);
            }
            Token::Comment { .. } => todo!(),
            Token::Character { data } if is_parser_whitespace(*data) => {
                // SPEC: Ignore the token.
            }
            Token::StartTag {
                name, attributes, ..
            } if name == "html" => {
                // SPEC: Create an element FIXME{for the token} FIXME{in the HTML namespace},
                //       FIXME{with the Document as the intended parent}.
                let dom_attributes = attributes
                    .iter()
                    .map(|a| dom::Attribute {
                        name: QualifiedName::new(None, (*a.name).to_string()),
                        value: (*a.value).to_string(),
                    })
                    .collect();

                let element = self
                    .tree_builder
                    .create_element(QualifiedName::new(None, name.to_string()), dom_attributes);

                // SPEC: Append it to the Document object.
                let document = self.tree_builder.document();
                self.tree_builder.append(&document, element.clone());

                // SPEC: Put this element in the stack of open elements.
                self.push_element_to_stack_of_open_elements(element);

                // SPEC: Switch the insertion mode to "before head".
                self.switch_insertion_mode_to(InsertionMode::BeforeHead);
            }
            Token::EndTag { name, .. }
                if name == "head" || name == "body" || name == "html" || name == "br" =>
            {
                todo!();
            }
            Token::EndTag { .. } => {
                todo!();
            }
            _ => {
                // SPEC: Create an html element FIXME{whose node document is the Document object}.
                let element = self
                    .tree_builder
                    .create_element(QualifiedName::new(None, "html".to_string()), Vec::new());

                // SPEC: Append it to the Document object.
                let document = self.tree_builder.document();
                self.tree_builder.append(&document, element.clone());

                // SPEC: Put this element in the stack of open elements.
                self.push_element_to_stack_of_open_elements(element);

                // SPEC: Switch the insertion mode to "before head", then reprocess the token.
                self.switch_insertion_mode_to(InsertionMode::BeforeHead);
                self.reprocess_token();
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-before-head-insertion-mode
    fn handle_before_head_insertion_mode(&mut self, _token: &Token) {
        // match token {
        //     Token::Character { data } if is_parser_whitespace(*data) => {
        //         // SPEC: Ignore the token.
        //     }
        //     Token::Comment { .. } => todo!(),
        //     Token::Doctype { .. } => todo!(),
        //     Token::StartTag { name, .. } if name == "html" => todo!(),
        //     Token::StartTag { name, .. } if name == "head" => {
        //         let token = token.clone();
        //         self.insert_html_element_for_token(&token);
        //     }
        //     Token::EndTag { name, .. } if name == "head" || name == "body" || name == "br" => {
        //         todo!()
        //     }
        //     Token::EndTag { .. } => {
        //         // SPEC: Parse error. Ignore the token.
        //     }
        //     _ => {
        //         // SPEC: Insert an HTML element for a "head" start tag token with no attributes.
        //         let element = self.insert_html_element_for_token(&Token::StartTag {
        //             name: String::from("head"),
        //             self_closing: false,
        //             self_closing_acknowledged: false,
        //             attributes: Vec::new(),
        //         });
        //         // SPEC: Set the head element pointer to the newly created head element.
        //         self.head_element = Some(element);

        //         // SPEC: Switch the insertion mode to "in head".
        //         self.switch_insertion_mode_to(InsertionMode::InHead);

        //         // SPEC: Reprocess the current token.
        //         self.reprocess_token();
        //     }
        // }
        todo!()
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inhead
    fn handle_in_head_insertion_mode(&mut self, _token: &Token) {
        // match token {
        //     Token::Character { data } if is_parser_whitespace(*data) => {
        //         // SPEC: Insert the character.
        //         self.insert_character(*data);
        //     }
        //     Token::Comment { data } => {
        //         // SPEC: Insert a comment.
        //         self.insert_comment(data, None);
        //     }
        //     Token::Doctype { .. } => {
        //         // SPEC: Parse error. Ignore the token.
        //     }
        //     Token::StartTag { name, .. } if name == "html" => {
        //         // SPEC: Process the token using the rules for the "in body" insertion mode.
        //         self.process_token_using_rules_of(InsertionMode::InBody)
        //     }
        //     Token::StartTag { name, .. }
        //         if name == "base" || name == "basefont" || name == "bgsound" || name == "link" =>
        //     {
        //         todo!()
        //     }
        //     Token::StartTag { name, .. } if name == "meta" => {
        //         todo!()
        //     }
        //     Token::StartTag { name, .. } if name == "title" => {
        //         // SPEC: Follow the generic RCDATA element parsing algorithm.
        //         self.follow_generic_parsing_algorithm(GenericParsingAlgorithm::RCData, token);
        //     }
        //     Token::StartTag { name, .. } if name == "noscript" && self.scripting_flag => {
        //         todo!()
        //     }
        //     Token::StartTag { name, .. } if name == "noframes" || name == "style" => {
        //         todo!()
        //     }
        //     Token::StartTag { name, .. } if name == "noscript" && self.scripting_flag => {
        //         todo!()
        //     }
        //     Token::StartTag { name, .. } if name == "script" => {
        //         todo!()
        //     }
        //     Token::EndTag { name, .. } if name == "head" => {
        //         todo!()
        //     }
        //     Token::EndTag { name, .. } if name == "body" || name == "html" || name == "br" => {
        //         todo!()
        //     }
        //     Token::StartTag { name, .. } if name == "template" => {
        //         todo!()
        //     }
        //     Token::StartTag { name, .. } if name == "head" => {
        //         todo!()
        //     }
        //     Token::EndTag { .. } => {
        //         // SPEC: Parse error. Ignore the token.
        //     }
        //     _ => {
        //         // SPEC: Pop the current node (which will be the head element) off the stack of open elements.
        //         self.pop_current_element_off_stack_of_open_elements();

        //         // SPEC: Switch the insertion mode to "after head".
        //         self.switch_insertion_mode_to(InsertionMode::AfterHead);

        //         // SPEC: Reprocess the token.
        //         self.reprocess_token();
        //     }
        // }
        todo!()
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-after-head-insertion-mode
    fn handle_after_head_insertion_mode(&mut self, _token: &Token) {
        // match token {
        //     Token::Character { data } if is_parser_whitespace(*data) => {
        //         // SPEC: Insert the character.
        //         self.insert_character(*data);
        //     }
        //     Token::Comment { data } => {
        //         // SPEC: Insert a comment.
        //         self.insert_comment(data, None);
        //     }
        //     Token::Doctype { .. } => {
        //         // SPEC: Parse error. Ignore the token.
        //     }
        //     Token::StartTag { name, .. } if name == "html" => {
        //         // SPEC: Process the token using the rules for the "in body" insertion mode.
        //         self.process_token_using_rules_of(InsertionMode::InBody);
        //     }
        //     Token::StartTag { name, .. } if name == "body" => {
        //         // SPEC: Insert an HTML element for the token.
        //         self.insert_html_element_for_token(token);

        //         // SPEC: Set the frameset-ok flag to "not ok".
        //         self.frameset_ok = FramesetState::NotOk;

        //         // SPEC: Switch the insertion mode to "in body".
        //         self.switch_insertion_mode_to(InsertionMode::InBody);
        //     }
        //     Token::StartTag { name, .. } if name == "frameset" => {
        //         // SPEC: Insert an HTML element for the token.
        //         self.insert_html_element_for_token(token);

        //         // SPEC: Switch the insertion mode to "in frameset".
        //         self.switch_insertion_mode_to(InsertionMode::InFrameset);
        //     }
        //     Token::StartTag { name, .. }
        //         if name == "base"
        //             || name == "basefont"
        //             || name == "bgsound"
        //             || name == "link"
        //             || name == "meta"
        //             || name == "noframes"
        //             || name == "script"
        //             || name == "style"
        //             || name == "template"
        //             || name == "title" =>
        //     {
        //         // SPEC: Parse error.

        //         // SPEC: Push the node pointed to by the head element pointer onto the stack of open elements.
        //         if let Some(head_element_pointer) = self.head_element.clone() {
        //             self.push_element_to_stack_of_open_elements(head_element_pointer);
        //         }

        //         // SPEC: Process the token using the rules for the "in head" insertion mode.
        //         self.process_token_using_rules_of(InsertionMode::InHead);

        //         // SPEC: Remove the node pointed to by the head element pointer from the stack of open elements.
        //         //       (It might not be the current node at this point.)
        //         if let Some(head_element_pointer) = self.head_element.clone() {
        //             self.remove_element_from_stack_of_open_elements(head_element_pointer);
        //         }
        //     }
        //     Token::EndTag { name, .. } if name == "template" => {
        //         // SPEC: Process the token using the rules for the "in head" insertion mode.
        //         self.process_token_using_rules_of(InsertionMode::InHead);
        //     }
        //     Token::EndTag { name, .. } if name == "body" || name == "html" || name == "br" => {
        //         // SPEC: Act as described in the "anything else" entry below.
        //     }
        //     Token::StartTag { name, .. } if name == "head" => {
        //         todo!()
        //     }
        //     Token::EndTag { .. } => {
        //         // SPEC: Parse error. Ignore the token.
        //     }
        //     _ => {
        //         // SPEC: Insert an HTML element for a "body" start tag token with no attributes.
        //         self.insert_html_element_for_token(&Token::StartTag {
        //             name: "body".to_string(),
        //             self_closing: false,
        //             self_closing_acknowledged: false,
        //             attributes: Vec::new(),
        //         });

        //         // SPEC: Switch the insertion mode to "in body".
        //         self.switch_insertion_mode_to(InsertionMode::InBody);

        //         // SPEC: Reprocess the current token.
        //         self.reprocess_token();
        //     }
        // }
        todo!()
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incdata
    fn handle_text_insertion_mode(&mut self, _token: &Token) {
        // match token {
        //     Token::Character { data } => {
        //         self.insert_character(*data);
        //     }
        //     Token::EndOfFile => {
        //         // SPEC: Parse error.

        //         // SPEC: If the current node is a script element, then set its already started to true.
        //         // FIXME: Implement

        //         // SPEC: Pop the current node off the stack of open elements.
        //         self.pop_current_element_off_stack_of_open_elements();

        //         // SPEC: Switch the insertion mode to the original insertion mode and reprocess the token.
        //         self.switch_insertion_mode_to(self.original_insertion_mode);
        //         self.reprocess_token();
        //     }
        //     Token::EndTag { name, .. } if name == "script" => todo!(),
        //     _ => {
        //         // SPEC: Pop the current node off the stack of open elements.
        //         self.pop_current_element_off_stack_of_open_elements();

        //         // SPEC: Switch the insertion mode to the original insertion mode.
        //         self.switch_insertion_mode_to(self.original_insertion_mode);
        //     }
        // }
        todo!()
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inforeign
    fn parse_token_in_foreign_content(&mut self, token: &mut Token) {
        // SPEC: When the user agent is to apply the rules for parsing tokens in foreign content, the user agent must handle the token as follows:
        match token {
            Token::Character { data } if data == &'\u{0000}' => {
                // SPEC: Parse error.
                self.tree_builder.parser_error(None);
                // SPEC: Insert a U+FFFD REPLACEMENT CHARACTER character.
                self.tree_builder.insert_character('\u{FFFD}');
            }
            Token::Character { data } if is_parser_whitespace(*data) => {
                // SPEC: Insert the token's character.
                self.tree_builder.insert_character(*data);
            }
            Token::Character { data } => {
                // SPEC: Insert the token's character.
                self.tree_builder.insert_character(*data);

                // Set the frameset-ok flag to "not ok".
                self.frameset_ok = FramesetState::NotOk;
            }
            Token::Comment { data } => {
                // SPEC: Insert a comment.
                self.tree_builder.insert_comment(data);
            }
            Token::Doctype { .. } => {
                // SPEC: Parse error. Ignore the token.
                self.tree_builder.parser_error(None);
            }
            // FIXME: Implement SPEC: A start tag whose tag name is one of: "b", "big", "blockquote", "body", "br", "center", "code", "dd", "div", "dl", "dt", "em", "embed", "h1", "h2", "h3", "h4", "h5", "h6", "head", "hr", "i", "img", "li", "listing", "menu", "meta", "nobr", "ol", "p", "pre", "ruby", "s", "small", "span", "strong", "strike", "sub", "sup", "table", "tt", "u", "ul", "var"
            //                        A start tag whose tag name is "font", if the token has any attributes named "color", "face", or "size"
            //                        An end tag whose tag name is "br", "p"
            Token::StartTag { self_closing, .. } => {
                // FIXME Implement SPEC: If the adjusted current node is an element in the MathML namespace, adjust MathML attributes for the token. (This fixes the case of MathML attributes that are not all lowercase.)
                // FIXME Implement SPEC: If the adjusted current node is an element in the SVG namespace, and the token's tag name is one of the ones in the first column of the following table, change the tag name to the name given in the corresponding cell in the second column. (This fixes the case of SVG elements that are not all lowercase.)
                // FIXME Implement SPEC: If the adjusted current node is an element in the SVG namespace, adjust SVG attributes for the token. (This fixes the case of SVG attributes that are not all lowercase.)
                // FIXME Implement SPEC: Adjust foreign attributes for the token. (This fixes the use of namespaced attributes, in particular XLink in SVG.)

                // SPEC: If the token has its self-closing flag set, then run the appropriate steps from the following list:
                if *self_closing {
                    // FIXME Implement SPEC: If the token's tag name is "script", and the new current node is in the SVG namespace
                    // SPEC: Acknowledge the token's self-closing flag, and then act as described in the steps for a "script" end tag below.
                    todo!()
                } else {
                    // SPEC: Otherwise
                    //       Pop the current node off the stack of open elements and acknowledge the token's self-closing flag.
                    token.acknowledge_self_closing_flag();
                    self.pop_current_element_off_stack_of_open_elements();
                }

                // NOTE: This has been reordered
                // SPEC: Insert a foreign element for the token, in the same namespace as the adjusted current node.
                // self.insert_foreign_element_for_token(token, None);
                todo!();
            }
            // FIXME: Implement SPEC: An end tag whose tag name is "script", if the current node is an SVG script element
            _ => {
                // FIXME Implement SPEC: 1. Initialize node to be the current node (the bottommost node of the stack).
                // FIXME Implement SPEC: 2. If node's tag name, converted to ASCII lowercase, is not the same as the tag name of the token, then this is a parse error.
                // FIXME Implement SPEC: 3. Loop: If node is the topmost element in the stack of open elements, then return. (fragment case)
                // FIXME Implement SPEC: 4. If node's tag name, converted to ASCII lowercase, is the same as the tag name of the token, pop elements from the stack of open elements until node has been popped from the stack, and then return.
                // FIXME Implement SPEC: 5. Set node to the previous entry in the stack of open elements.
                // FIXME Implement SPEC: 6. If node is not an element in the HTML namespace, return to the step labeled loop.
                // FIXME Implement SPEC: 7. Otherwise, process the token according to the rules given in the section corresponding to the current insertion mode in HTML content.
            }
        }
    }

    pub fn parse(&mut self) -> T::Handle {
        while let Some(token) = match self.reprocess_current_token {
            true => self.tokenizer.current_token(),
            false => self.tokenizer.next_token(),
        } {
            // eprintln!("[{:?}] {:?}", self.insertion_mode, token);

            let mut token = token.clone();

            // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#tree-construction-dispatcher
            // As each token is emitted from the tokenizer,
            // the user agent must follow the appropriate steps from the following list,
            // known as the tree construction dispatcher:

            // SPEC: If the stack of open elements is empty
            // FIXME If the adjusted current node is an element in the HTML namespace
            // FIXME If the adjusted current node is a MathML text integration point and the token is a start tag whose tag name is neither "mglyph" nor "malignmark"
            // FIXME If the adjusted current node is a MathML text integration point and the token is a character token
            // FIXME If the adjusted current node is a MathML annotation-xml element and the token is a start tag whose tag name is "svg"
            // FIXME If the adjusted current node is an HTML integration point and the token is a start tag
            // FIXME If the adjusted current node is an HTML integration point and the token is a character token
            //       If the token is an end-of-file token
            if self.open_elements.is_empty()
                || match token {
                    Token::EndOfFile => true,
                    _ => false,
                }
            {
                self.process_token(&token);
            } else {
                self.parse_token_in_foreign_content(&mut token)
            }

            self.reprocess_current_token = false;
        }

        self.tree_builder.document()
    }
}
