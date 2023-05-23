use std::cell::RefCell;

use dom::arena::{Arena, Ref};
use dom::node::{Node, NodeData};
use dom::{Namespace, QualifiedName};
use tokenizer::{Token, Tokenizer};

const fn is_parser_whitespace(string: char) -> bool {
    if let '\t' | '\u{000a}' | '\u{000c}' | '\u{000d}' | '\u{0020}' = string {
        return true;
    }
    false
}

const SPECIAL_TAGS: &[&str] = &[
    "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc", "tbody", "td", "tfoot",
    "th", "thead", "tr", "body", "html",
];

macro_rules! log_parser_error {
    ($message:expr) => {
        eprintln!("Parser Error on {}:{}: {}", file!(), line!(), $message);
    };
    () => {
        eprintln!("Parser Error on {}:{}", file!(), line!());
    };
}

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

#[derive(Clone)]
pub struct Parser<'arena> {
    arena: Arena<'arena>,
    document: Ref<'arena>,

    insertion_mode: InsertionMode,
    original_insertion_mode: InsertionMode,
    referenced_insertion_mode: Option<InsertionMode>,
    tokenizer: Tokenizer,
    reprocess_current_token: bool,
    open_elements: Vec<Ref<'arena>>,
    head_element: Option<Ref<'arena>>,
    foster_parenting: bool,
    scripting_flag: bool,
    frameset_ok: FramesetState,
}

impl<'arena> Parser<'arena> {
    pub fn new(arena: Arena<'arena>, input: &str) -> Self {
        Self {
            arena,
            document: arena.alloc(Node::new(None, NodeData::Document)),
            insertion_mode: InsertionMode::Initial,
            original_insertion_mode: InsertionMode::Initial,
            referenced_insertion_mode: None,
            tokenizer: Tokenizer::new(input),
            reprocess_current_token: false,
            open_elements: Vec::new(),
            head_element: None,
            foster_parenting: false,
            scripting_flag: false,
            frameset_ok: FramesetState::Ok,
        }
    }

    fn new_node(&self, document: Ref<'arena>, data: NodeData) -> Ref<'arena> {
        self.arena.alloc(Node::new(Some(document), data))
    }

    fn push_element_to_stack_of_open_elements(&mut self, element: Ref<'arena>) {
        self.open_elements.push(element);
    }

    fn pop_current_element_off_stack_of_open_elements(&mut self) {
        self.open_elements.pop();
    }

    fn pop_elements_from_stack_of_open_elements_until_element_has_been_popped(
        &mut self,
        tag_name: &str,
    ) {
        let mut current = self.current_node();
        while let Some(NodeData::Element { name, .. }) = current.map(|c| &c.data) {
            self.pop_current_element_off_stack_of_open_elements();
            if name.local == tag_name {
                return;
            }
            current = self.current_node();
        }
    }

    fn remove_element_from_stack_of_open_elements(&mut self, element: Ref<'arena>) {
        if let Some(index) = self.open_elements.iter().position(|e| e == &element) {
            self.open_elements.remove(index);
        }
    }

    fn stack_of_open_elements_contains_one_of(&self, names: &[&str]) -> bool {
        self.open_elements.iter().any(|element| {
            if let NodeData::Element { name, .. } = &element.data {
                return names.contains(&name.local.as_str());
            }
            false
        })
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#generate-implied-end-tags
    fn generate_implied_end_tags_except_for(&mut self, except_for: &str) {
        // SPEC: while the current node is a dd element, a dt element, an li element, an optgroup element,
        //       an option element, a p element, an rb element, an rp element, an rt element, or an rtc element,
        //       the UA must pop the current node off the stack of open elements.
        let mut current = self.current_node();
        while let Some(NodeData::Element { name, .. }) = current.map(|c| &c.data) {
            if name.local.as_str() == except_for {
                break;
            }
            if [
                "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc",
            ]
            .contains(&name.local.as_str())
            {
                return;
            }
            self.pop_current_element_off_stack_of_open_elements();
            current = self.current_node();
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
            InsertionMode::Initial => self.handle_initial(token),
            InsertionMode::BeforeHtml => self.handle_before_html(token),
            InsertionMode::BeforeHead => self.handle_before_head(token),
            InsertionMode::InHead => self.handle_in_head(token),
            InsertionMode::InHeadNoscript => todo!("InsertionMode::InHeadNoscript"),
            InsertionMode::AfterHead => self.handle_after_head(token),
            InsertionMode::InBody => self.handle_in_body(token),
            InsertionMode::Text => self.handle_text(token),
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
            InsertionMode::AfterBody => self.handle_after_body(token),
            InsertionMode::InFrameset => todo!("InsertionMode::InFrameset"),
            InsertionMode::AfterFrameset => todo!("InsertionMode::AfterFrameset"),
            InsertionMode::AfterAfterBody => self.handle_after_after_body(token),
            InsertionMode::AfterAfterFrameset => todo!("InsertionMode::AfterAfterFrameset"),
        }
    }

    fn process_token_using_the_rules_for(&mut self, insertion_mode: InsertionMode) {
        self.referenced_insertion_mode = Some(insertion_mode);
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#current-node
    fn current_node(&self) -> Option<Ref<'arena>> {
        // SPEC: The current node is the bottommost node in this stack of open elements.
        self.open_elements.last().cloned()
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#adjusted-current-node
    fn adjusted_current_node(&self) -> Option<Ref<'arena>> {
        // FIXME: Implement
        self.current_node()
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#appropriate-place-for-inserting-a-node
    fn appropriate_place_for_inserting_node(
        &self,
        override_target: Option<Ref<'arena>>,
    ) -> (Option<Ref<'arena>>, Option<Ref<'arena>>) {
        let target = match override_target {
            // SPEC: 1. If there was an override target specified, then let target be the override target.
            Some(override_target) => Some(override_target),
            // SPEC: Otherwise, let target be the current node.
            None => self.current_node(),
        };

        // SPEC: 2. Determine the adjusted insertion location using the first matching steps from the following list:
        let adjusted_insertion_location = if self.foster_parenting {
            // SPEC: If foster parenting is enabled and target is a table, tbody, tfoot, thead, or tr element
            todo!()
        } else {
            // SPEC: Otherwise, let adjusted insertion location be inside target, after its last child (if any).
            (target, None)
        };

        // SPEC: If the adjusted insertion location is inside a template element,
        //       let it instead be inside the template element's template contents, after its last child (if any).
        // FIXME: Implement

        // SPEC: 4. Return the adjusted insertion location.
        #[allow(clippy::let_and_return)]
        adjusted_insertion_location
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-create-element
    fn create_element(
        &mut self,
        document: Ref<'arena>,
        name: QualifiedName,
        attributes: Vec<dom::Attribute>,
    ) -> Ref<'arena> {
        // FIXME: This does not implement any spec functionallity yet!

        let element = self.new_node(
            document,
            NodeData::Element {
                name,
                attributes: RefCell::new(attributes),
            },
        );

        element
    }

    fn append_doctype_to_document(&mut self, name: &str, public_id: &str, system_id: &str) {
        let node = self.new_node(
            self.document,
            NodeData::Doctype {
                name: name.to_string(),
                public_id: public_id.to_string(),
                system_id: system_id.to_string(),
            },
        );

        self.document.append(node);
    }

    fn find_character_insertion_node(&self) -> Option<Ref<'arena>> {
        let adjusted_insertion_location = self.appropriate_place_for_inserting_node(None);

        if adjusted_insertion_location.1.is_some() {
            todo!()
        }

        if adjusted_insertion_location.0?.is_document() {
            return None;
        }

        if let Some(text_node) = adjusted_insertion_location.0?.last_child() {
            return Some(text_node);
        }

        let new_text_node = self.new_node(
            self.document,
            NodeData::Text {
                contents: RefCell::new(String::new()),
            },
        );

        adjusted_insertion_location.0?.append(new_text_node);

        Some(new_text_node)
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#reconstruct-the-active-formatting-elements
    fn reconstruct_active_formatting_elements_if_any(&mut self) {
        // FIXME: Implement
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#close-a-p-element
    fn close_a_p_element(&mut self) {
        // SPEC: Generate implied end tags, except for p elements.
        self.generate_implied_end_tags_except_for("p");

        // SPEC: If the current node is not a p element, then this is a parse error.
        if let Some(NodeData::Element { name, .. }) = self.current_node().map(|c| &c.data) {
            if name.local != "p" {
                log_parser_error!();
            }
        }
        // SPEC: Pop elements from the stack of open elements until a p element has been popped from the stack.
        self.pop_elements_from_stack_of_open_elements_until_element_has_been_popped("p");
    }

    // SPECLINK: https://html.spec.whatwg.org/#insert-a-character
    fn insert_character(&mut self, data: char) {
        if let Some(NodeData::Text { contents }) =
            self.find_character_insertion_node().map(|node| &node.data)
        {
            contents.borrow_mut().push(data);
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/#insert-a-comment
    fn insert_comment(&self, _data: &str) {
        todo!()
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#insert-an-html-element
    fn insert_html_element_for_token(&mut self, token: &Token) -> Ref<'arena> {
        self.insert_foreign_element_for_token(token, None)
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#insert-a-foreign-element
    fn insert_foreign_element_for_token(
        &mut self,
        token: &Token,
        _namespace: Option<&str>,
    ) -> Ref<'arena> {
        // SPEC: 1. Let the adjusted insertion location be the appropriate place for inserting a node.
        let adjusted_insertion_location = self.appropriate_place_for_inserting_node(None);

        // SPEC: 2. Let element be the result of creating an element for the token in the given namespace,
        //          with the intended parent being the element in which the adjusted insertion location finds itself.

        let element = self.create_element_for_token(token, adjusted_insertion_location.0.unwrap());

        let pre_insertion_validity = adjusted_insertion_location
            .0
            .unwrap()
            .ensure_pre_insertion_validity(element, adjusted_insertion_location.1);

        // SPEC: 3. If it is possible to insert element at the adjusted insertion location, then:
        if pre_insertion_validity.is_ok() {
            // SPEC: 3.1. If the parser was not created as part of the HTML fragment parsing algorithm,
            //            then push a new element queue onto element's relevant agent's custom element reactions stack.
            // FIXME: Implement

            // SPEC: 3.2. Insert element at the adjusted insertion location.
            adjusted_insertion_location
                .0
                .unwrap()
                .insert(element, adjusted_insertion_location.1);

            // SPEC: 3.3. If the parser was not created as part of the HTML fragment parsing algorithm,
            //            then pop the element queue from element's relevant agent's custom element reactions stack,
            //            and invoke custom element reactions in that queue.
            // FIXME: Implement
        }

        // SPEC: 4. Push element onto the stack of open elements so that it is the new current node.
        self.push_element_to_stack_of_open_elements(element);

        // SPEC: 5. Return element.
        element
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#create-an-element-for-the-token
    fn create_element_for_token(
        &mut self,
        token: &Token,
        intended_parent: Ref<'arena>,
    ) -> Ref<'arena> {
        // SPEC: 1. If the active speculative HTML parser is not null,
        //          then return the result of creating a speculative mock element given given namespace,
        //          the tag name of the given token,
        //          and the attributes of the given token.
        // FIXME: Implement

        // SPEC: 2. Otherwise, optionally create a speculative mock element given given namespace,
        //          the tag name of the given token,
        //          and the attributes of the given token.
        // FIXME: Implement

        // SPEC: 3. Let document be intended parent's node document.
        let document = intended_parent.document();

        // SPEC: 4. Let local name be the tag name of the token.
        let (local_name, attributes) = match token {
            Token::StartTag {
                name: local_name,
                attributes,
                ..
            } => {
                // SPEC: 5. Let is be the value of the "is" attribute in the given token,
                //          if such an attribute exists,
                //          or null otherwise.
                // FIXME: Implement

                // SPEC: 6. Let definition be the result of looking up a custom element definition
                //          given document, given namespace, local name, and is.
                // FIXME: Implement

                // SPEC: 7. If definition is non-null and the parser was not created as part of the HTML fragment parsing algorithm,
                //          then let will execute script be true.
                //          Otherwise, let it be false.
                // FIXME: Implement

                // SPEC: 8. If will execute script is true, then:
                // FIXME: Implement

                // FIXME: Use impl From for this
                let dom_attributes = attributes
                    .iter()
                    .map(|attr| dom::Attribute {
                        name: QualifiedName::new(attr.name.clone()),
                        value: attr.value.clone(),
                    })
                    .collect();

                (local_name, dom_attributes)
            }
            _ => panic!("cannot create element from non-StartTag token"),
        };

        // SPEC: 9. Let element be the result of creating an element
        //          given document, localName, FIXME{given namespace, null, and is.}
        //          FIXME{If will execute script is true, set the synchronous custom elements flag; otherwise, leave it unset.}
        // SPEC: 10. Append each attribute in the given token to element.
        let element = self.create_element(
            document,
            QualifiedName::new(local_name.to_owned()),
            attributes,
        );

        // SPEC: 11. If will execute script is true, then:
        // FIXME: Implement

        // SPEC: 12. If element has an xmlns attribute in the XMLNS namespace whose value
        //           is not exactly the same as the element's namespace, that is a parse error.
        //           Similarly, if element has an xmlns:xlink attribute in the XMLNS namespace
        //           whose value is not the XLink Namespace, that is a parse error.
        // FIXME: Implement

        // SPEC: 13. If element is a resettable element,
        //           invoke its reset algorithm.
        //           (This initializes the element's value and checkedness based on the element's attributes.)
        // FIXME: Implement

        // SPEC: 14. If element is a form-associated element and not a form-associated custom element,
        //           the form element pointer is not null,
        //           there is no template element on the stack of open elements,
        //           element is either not listed or doesn't have a form attribute,
        //           and the intended parent is in the same tree as the element pointed to by the form element pointer,
        //           then associate element with the form element pointed to by the form element pointer and set element's parser inserted flag.
        // FIXME: Implement

        // SPEC: 15. Return element.
        element
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#generic-rcdata-element-parsing-algorithm
    fn follow_generic_parsing_algorithm(
        &mut self,
        algorithm: GenericParsingAlgorithm,
        token: &Token,
    ) {
        // SPEC: 1. Insert an HTML element for the token.
        self.insert_html_element_for_token(token);

        // SPEC: 2. If the algorithm that was invoked is the generic raw text element parsing algorithm,
        //          switch the tokenizer to the RAWTEXT state;
        //          otherwise the algorithm invoked was the generic RCDATA element parsing algorithm,
        //          switch the tokenizer to the RCDATA state.
        match algorithm {
            GenericParsingAlgorithm::RawText => todo!(),
            GenericParsingAlgorithm::RCData => {
                self.tokenizer.switch_to(tokenizer::State::RcData);
            }
        }

        // SPEC: 3. Let the original insertion mode be the current insertion mode.
        self.original_insertion_mode = self.insertion_mode;
        // SPEC: 4. Then, switch the insertion mode to "text".
        self.switch_insertion_mode_to(InsertionMode::Text);
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode
    fn handle_initial(&mut self, token: &Token) {
        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                // SPEC: Ignore the token.
            }
            Token::Comment { data } => {
                // SPEC: Insert a comment as the last child of the Document object.
                self.insert_comment(data);
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
                    || public_identifier.is_some()
                    || (system_identifier.is_some()
                        && system_identifier != &Some("about:legacy-compat".to_string()))
                {
                    // SPEC: then there is a parse error.
                    log_parser_error!();
                }

                // SPEC: Append a DocumentType node to the Document node,
                //       with its name set to the name given in the DOCTYPE token, or the empty string if the name was missing;
                //       its public ID set to the public identifier given in the DOCTYPE token, or the empty string if the public identifier was missing;
                //       and its system ID set to the system identifier given in the DOCTYPE token, or the empty string if the system identifier was missing.
                self.append_doctype_to_document(
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
    fn handle_before_html(&mut self, token: &Token) {
        match token {
            Token::Doctype { .. } => {
                // SPEC: Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::Comment { .. } => todo!(),
            Token::Character { data } if is_parser_whitespace(*data) => {
                // SPEC: Ignore the token.
            }
            Token::StartTag { name, .. } if name == "html" => {
                // SPEC: Create an element for the token in the FIXME{HTML namespace},
                //       with the Document as the intended parent.
                let element = self.create_element_for_token(token, self.document);

                // SPEC: Append it to the Document object.
                self.document.append(element);

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
                // SPEC: Create an html element whose node document is the Document object.
                let element = self.create_element(
                    self.document,
                    QualifiedName::new("html".to_string()),
                    Vec::new(),
                );

                // SPEC: Append it to the Document object.
                self.document.append(element);

                // SPEC: Put this element in the stack of open elements.
                self.push_element_to_stack_of_open_elements(element);

                // SPEC: Switch the insertion mode to "before head", then reprocess the token.
                self.switch_insertion_mode_to(InsertionMode::BeforeHead);
                self.reprocess_token();
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-before-head-insertion-mode
    fn handle_before_head(&mut self, token: &Token) {
        let mut create_head = || {
            // SPEC: Insert an HTML element for a "head" start tag token with no attributes.
            let element = self.insert_html_element_for_token(&Token::StartTag {
                name: String::from("head"),
                self_closing: false,
                self_closing_acknowledged: false,
                attributes: Vec::new(),
            });
            // SPEC: Set the head element pointer to the newly created head element.
            self.head_element = Some(element);

            // SPEC: Switch the insertion mode to "in head".
            self.switch_insertion_mode_to(InsertionMode::InHead);

            // SPEC: Reprocess the current token.
            self.reprocess_token();
        };

        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                // SPEC: Ignore the token.
            }
            Token::Comment { .. } => todo!(),
            Token::Doctype { .. } => todo!(),
            Token::StartTag { name, .. } if name == "html" => todo!(),
            Token::StartTag { name, .. } if name == "head" => {
                // SPEC: Insert an HTML element for the token.
                let element = self.insert_html_element_for_token(token);

                // SPEC: Set the head element pointer to the newly created head element.
                self.head_element = Some(element);

                // SPEC: Switch the insertion mode to "in head".
                self.switch_insertion_mode_to(InsertionMode::InHead);
            }
            Token::EndTag { name, .. } if name == "head" || name == "body" || name == "br" => {
                // SPEC: Act as described in the "anything else" entry below.
                create_head()
            }
            Token::EndTag { name, .. } => {
                // SPEC: Parse error. Ignore the token.
                log_parser_error!(format!("Invalid End Tag: {name}"));
            }
            _ => create_head(),
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inhead
    fn handle_in_head(&mut self, token: &Token) {
        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                // SPEC: Insert the character.
                self.insert_character(*data);
            }
            Token::Comment { data } => {
                // SPEC: Insert a comment.
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {
                // SPEC: Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::StartTag { name, .. } if name == "html" => {
                // SPEC: Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody)
            }
            Token::StartTag { name, .. }
                if name == "base" || name == "basefont" || name == "bgsound" || name == "link" =>
            {
                todo!()
            }
            Token::StartTag { name, .. } if name == "meta" => {
                todo!()
            }
            Token::StartTag { name, .. } if name == "title" => {
                // SPEC: Follow the generic RCDATA element parsing algorithm.
                self.follow_generic_parsing_algorithm(GenericParsingAlgorithm::RCData, token);
            }
            Token::StartTag { name, .. } if name == "noscript" && self.scripting_flag => {
                todo!()
            }
            Token::StartTag { name, .. } if name == "noframes" || name == "style" => {
                todo!()
            }
            Token::StartTag { name, .. } if name == "noscript" && self.scripting_flag => {
                todo!()
            }
            Token::StartTag { name, .. } if name == "script" => {
                todo!()
            }
            Token::EndTag { name, .. } if name == "head" => {
                // SPEC: Pop the current node (which will be the head element) off the stack of open elements.
                self.pop_current_element_off_stack_of_open_elements();

                // SPEC: Switch the insertion mode to "after head".
                self.switch_insertion_mode_to(InsertionMode::AfterHead);
            }
            Token::EndTag { name, .. } if name == "body" || name == "html" || name == "br" => {
                todo!()
            }
            Token::StartTag { name, .. } if name == "template" => {
                todo!()
            }
            Token::StartTag { name, .. } if name == "head" => {
                todo!()
            }
            Token::EndTag { .. } => {
                // SPEC: Parse error. Ignore the token.
                log_parser_error!();
            }
            _ => {
                // SPEC: Pop the current node (which will be the head element) off the stack of open elements.
                self.pop_current_element_off_stack_of_open_elements();

                // SPEC: Switch the insertion mode to "after head".
                self.switch_insertion_mode_to(InsertionMode::AfterHead);

                // SPEC: Reprocess the token.
                self.reprocess_token();
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-after-head-insertion-mode
    fn handle_after_head(&mut self, token: &Token) {
        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                // SPEC: Insert the character.
                self.insert_character(*data);
            }
            Token::Comment { data } => {
                // SPEC: Insert a comment.
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {
                // SPEC: Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::StartTag { name, .. } if name == "html" => {
                // SPEC: Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody);
            }
            Token::StartTag { name, .. } if name == "body" => {
                // SPEC: Insert an HTML element for the token.
                self.insert_html_element_for_token(token);

                // SPEC: Set the frameset-ok flag to "not ok".
                self.frameset_ok = FramesetState::NotOk;

                // SPEC: Switch the insertion mode to "in body".
                self.switch_insertion_mode_to(InsertionMode::InBody);
            }
            Token::StartTag { name, .. } if name == "frameset" => {
                // SPEC: Insert an HTML element for the token.
                self.insert_html_element_for_token(token);

                // SPEC: Switch the insertion mode to "in frameset".
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
                // SPEC: Parse error.
                log_parser_error!();

                // SPEC: Push the node pointed to by the head element pointer onto the stack of open elements.
                if let Some(head_element_pointer) = self.head_element {
                    self.push_element_to_stack_of_open_elements(head_element_pointer);
                }

                // SPEC: Process the token using the rules for the "in head" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InHead);

                // SPEC: Remove the node pointed to by the head element pointer from the stack of open elements.
                //       (It might not be the current node at this point.)
                if let Some(head_element_pointer) = self.head_element {
                    self.remove_element_from_stack_of_open_elements(head_element_pointer);
                }
            }
            Token::EndTag { name, .. } if name == "template" => {
                // SPEC: Process the token using the rules for the "in head" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InHead);
            }
            Token::EndTag { name, .. } if name == "body" || name == "html" || name == "br" => {
                // SPEC: Act as described in the "anything else" entry below.
            }
            Token::StartTag { name, .. } if name == "head" => {
                todo!()
            }
            Token::EndTag { .. } => {
                // SPEC: Parse error. Ignore the token.
                log_parser_error!();
            }
            _ => {
                // SPEC: Insert an HTML element for a "body" start tag token with no attributes.
                self.insert_html_element_for_token(&Token::StartTag {
                    name: "body".to_string(),
                    self_closing: false,
                    self_closing_acknowledged: false,
                    attributes: Vec::new(),
                });

                // SPEC: Switch the insertion mode to "in body".
                self.switch_insertion_mode_to(InsertionMode::InBody);

                // SPEC: Reprocess the current token.
                self.reprocess_token();
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody
    fn handle_in_body(&mut self, token: &Token) {
        match token {
            Token::Character { data } if data == &'\u{0000}' => {
                // SPEC: Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::Character { data } if is_parser_whitespace(*data) => {
                // SPEC: Reconstruct the active formatting elements, if any.
                self.reconstruct_active_formatting_elements_if_any();

                // SPEC: Insert the token's character.
                self.insert_character(*data);
            }
            Token::Character { data } => {
                // SPEC: Reconstruct the active formatting elements, if any.
                self.reconstruct_active_formatting_elements_if_any();
                // SPEC: Insert the token's character.
                self.insert_character(*data);
                // SPEC: Set the frameset-ok flag to "not ok".
                self.frameset_ok = FramesetState::NotOk;
            }
            Token::Comment { data } => {
                // SPEC: Insert a comment.
                self.insert_comment(data)
            }
            Token::Doctype { .. } => {
                // SPEC: Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::StartTag { name, .. } if name == "html" => todo!(),
            // FIXME: A start tag whose tag name is one of: "base", "basefont", "bgsound", "link", "meta", "noframes", "script", "style", "template", "title"
            //        An end tag whose tag name is "template"
            Token::StartTag { name, .. } if name == "body" => todo!(),
            Token::StartTag { name, .. } if name == "frameset" => todo!(),
            Token::EndOfFile => {
                // SPEC: If the stack of template insertion modes is not empty, then process the token using the rules for the "in template" insertion mode.
                // FIXME: Implement

                // Otherwise, follow these steps:

                // If there is a node in the stack of open elements that is not either a dd element, a dt element, an li element, an optgroup element, an option element, a p element, an rb element, an rp element, an rt element, an rtc element, a tbody element, a td element, a tfoot element, a th element, a thead element, a tr element, the body element, or the html element, then this is a parse error.
                if !self.stack_of_open_elements_contains_one_of(SPECIAL_TAGS) {
                    log_parser_error!();
                };

                // Stop parsing.
                self.stop_parsing();
            }
            Token::EndTag { name, .. } if name == "body" => {
                // SPEC: If the stack of open elements does not have a body element in scope, this is a parse error; ignore the token.
                if !self.stack_of_open_elements_contains_one_of(&["body"]) {
                    log_parser_error!();
                    return;
                }

                // SPEC: Otherwise, if there is a node in the stack of open elements that is not either a
                //       dd element, a dt element, an li element, an optgroup element, an option element, a p element, an rb element,
                //       an rp element, an rt element, an rtc element, a tbody element, a td element, a tfoot element, a th element,
                //       a thead element, a tr element, the body element, or the html element, then this is a parse error.
                if !self.stack_of_open_elements_contains_one_of(SPECIAL_TAGS) {
                    log_parser_error!();
                }

                // SPEC: Switch the insertion mode to "after body".
                self.switch_insertion_mode_to(InsertionMode::AfterBody);
            }
            Token::EndTag { name, .. } if name == "html" => {
                // SPEC: 1. If the stack of open elements does not have a body element in scope,
                //          this is a parse error; ignore the token.
                if !self.stack_of_open_elements_contains_one_of(&["body"]) {
                    log_parser_error!();
                    return;
                }
                // SPEC: 2. Otherwise, if there is a node in the stack of open elements that is not either a dd element, a dt element, an li element, an optgroup element, an option element, a p element, an rb element, an rp element, an rt element, an rtc element, a tbody element, a td element, a tfoot element, a th element, a thead element, a tr element, the body element, or the html element, then this is a parse error.

                // SPEC: 3. Switch the insertion mode to "after body".
                self.switch_insertion_mode_to(InsertionMode::AfterBody);
                // SPEC: 4. Reprocess the token.
                self.reprocess_token();
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
                // SPEC: If the stack of open elements has a p element in button scope, then close a p element.
                if self.stack_of_open_elements_contains_one_of(&["p"]) {
                    self.close_a_p_element();
                }
                // SPEC: Insert an HTML element for the token.
                self.insert_html_element_for_token(token);
            }
            Token::EndTag { name, .. } if name == "p" => {
                // SPEC: If the stack of open elements does not have a p element in button scope, then this is a parse error;
                //       insert an HTML element for a "p" start tag token with no attributes.
                if !self.stack_of_open_elements_contains_one_of(&["p"]) {
                    log_parser_error!();
                    self.insert_html_element_for_token(&Token::StartTag {
                        name: String::from("p"),
                        self_closing: false,
                        self_closing_acknowledged: false,
                        attributes: Vec::new(),
                    });
                }

                // SPEC: Close a p element.
                self.close_a_p_element();
            }
            _ => todo!("{token:?}"),
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incdata
    fn handle_text(&mut self, token: &Token) {
        match token {
            Token::Character { data } => {
                // SPEC: Insert the token's character.
                self.insert_character(*data);
            }
            Token::EndOfFile => {
                // SPEC: Parse error.
                log_parser_error!();

                // SPEC: If the current node is a script element, then set its already started to true.
                // FIXME: Implement

                // SPEC: Pop the current node off the stack of open elements.
                self.pop_current_element_off_stack_of_open_elements();

                // SPEC: Switch the insertion mode to the original insertion mode and reprocess the token.
                self.switch_insertion_mode_to(self.original_insertion_mode);
                self.reprocess_token();
            }
            Token::EndTag { name, .. } if name == "script" => todo!(),
            _ => {
                // SPEC: Pop the current node off the stack of open elements.
                self.pop_current_element_off_stack_of_open_elements();

                // SPEC: Switch the insertion mode to the original insertion mode.
                self.switch_insertion_mode_to(self.original_insertion_mode);
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-afterbody
    fn handle_after_body(&mut self, token: &Token) {
        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                // SPEC: Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody);
            }
            Token::Comment { .. } => todo!(),
            Token::Doctype { .. } => {
                // SPEC: Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::StartTag { name, .. } if name == "html" => {
                // SPEC: Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody);
            }
            Token::EndTag { name, .. } if name == "html" => {
                // SPEC: If the parser was created as part of the HTML fragment parsing algorithm,
                //       this is a parse error; ignore the token. (fragment case)
                // FIXME: Implement

                // SPEC: Otherwise, switch the insertion mode to "after after body".
                self.switch_insertion_mode_to(InsertionMode::AfterAfterBody);
            }
            Token::EndOfFile => {
                // SPEC: Stop parsing.
                self.stop_parsing();
            }
            _ => {
                // SPEC: Parse error.
                log_parser_error!();

                // SPEC: Switch the insertion mode to "in body" and reprocess the token.
                self.switch_insertion_mode_to(InsertionMode::InBody);
                self.reprocess_token();
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-after-after-body-insertion-mode
    fn handle_after_after_body(&mut self, token: &Token) {
        let mut process_token = || {
            // SPEC: Process the token using the rules for the "in body" insertion mode.
            self.process_token_using_the_rules_for(InsertionMode::InBody);
        };

        match token {
            Token::Comment { .. } => todo!(),
            Token::Doctype { .. } => process_token(),
            Token::Character { data } if is_parser_whitespace(*data) => process_token(),
            Token::StartTag { name, .. } if name == "html" => process_token(),
            Token::EndOfFile => {
                // SPEC: Stop parsing.
                self.stop_parsing()
            }
            _ => {
                // SPEC: Parse error.
                log_parser_error!();

                // SPEC: Switch the insertion mode to "in body" and reprocess the token.
                self.switch_insertion_mode_to(InsertionMode::InBody);
                self.reprocess_token();
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inforeign
    fn process_token_using_the_rules_for_foreign_content(&mut self, token: &mut Token) {
        // SPEC: When the user agent is to apply the rules for parsing tokens in foreign content, the user agent must handle the token as follows:
        match token {
            Token::Character { data } if data == &'\u{0000}' => {
                // SPEC: Parse error.
                log_parser_error!();

                // SPEC: Insert a U+FFFD REPLACEMENT CHARACTER character.
                self.insert_character('\u{FFFD}');
            }
            Token::Character { data } if is_parser_whitespace(*data) => {
                // SPEC: Insert the token's character.
                self.insert_character(*data);
            }
            Token::Character { data } => {
                // SPEC: Insert the token's character.
                self.insert_character(*data);

                // Set the frameset-ok flag to "not ok".
                self.frameset_ok = FramesetState::NotOk;
            }
            Token::Comment { data } => {
                // SPEC: Insert a comment.
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {
                // SPEC: Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::StartTag {
                name, attributes, ..
            } if name == "b"
                || name == "big"
                || name == "blockquote"
                || name == "body"
                || name == "br"
                || name == "center"
                || name == "code"
                || name == "dd"
                || name == "div"
                || name == "dl"
                || name == "dt"
                || name == "em"
                || name == "embed"
                || name == "h1"
                || name == "h2"
                || name == "h3"
                || name == "h4"
                || name == "h5"
                || name == "h6"
                || name == "head"
                || name == "hr"
                || name == "i"
                || name == "img"
                || name == "li"
                || name == "listing"
                || name == "menu"
                || name == "meta"
                || name == "nobr"
                || name == "ol"
                || name == "p"
                || name == "pre"
                || name == "ruby"
                || name == "s"
                || name == "small"
                || name == "span"
                || name == "strong"
                || name == "strike"
                || name == "sub"
                || name == "sup"
                || name == "table"
                || name == "tt"
                || name == "u"
                || name == "ul"
                || name == "var"
                || (name == "font"
                    && attributes.iter().any(|attr| {
                        attr.name == "color" || attr.name == "face" || attr.name == "size"
                    })) =>
            // FIXME: An end tag whose tag name is "br", "p"
            {
                // SPEC: Parse error.
                log_parser_error!(format!("Invalid StartTag token: {name}"));

                // SPEC: While the current node is not a MathML text integration point, an HTML integration point, or an element in the HTML namespace, pop elements from the stack of open elements.
                // FIXME: Implement

                // SPEC: Reprocess the token according to the rules given in the section corresponding to the current insertion mode in HTML content.
                self.reprocess_token();
            }
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
                self.insert_foreign_element_for_token(token, None);
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

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#stop-parsing
    fn stop_parsing(&mut self) {
        // SPEC: 1. If the active speculative HTML parser is not null, then stop the speculative HTML parser and return.
        // FIXME: Implement
        // SPEC: 2. Set the insertion point to undefined.
        self.tokenizer.set_insertion_point(None);
        // SPEC: 3. Update the current document readiness to "interactive".
        // FIXME: Implement
        // SPEC: 4. Pop all the nodes off the stack of open elements.
        self.open_elements.clear();
        // SPEC: 5. While the list of scripts that will execute when the document has finished parsing is not empty:
        // FIXME: Implement
        // SPEC: 6. Queue a global task on the DOM manipulation task source given the Document's relevant global object to run the following substeps:
        // FIXME: Implement
        // SPEC: 7. Spin the event loop until the set of scripts that will execute as soon as possible and the list of scripts that will execute in order as soon as possible are empty.
        // FIXME: Implement
        // SPEC: 8. Spin the event loop until there is nothing that delays the load event in the Document.
        // FIXME: Implement
        // SPEC: 9. Queue a global task on the DOM manipulation task source given the Document's relevant global object to run the following steps:
        // SPEC: 10. If the Document's print when loaded flag is set, then run the printing steps.
        // FIXME: Implement
        // SPEC: 11. The Document is now ready for post-load tasks.
        // FIXME: Implement
    }

    pub fn parse(&mut self) -> Ref<'arena> {
        while let Some(token) = match self.reprocess_current_token {
            true => self.tokenizer.current_token(),
            false => self.tokenizer.next_token(),
        } {
            eprintln!(
                "\x1b[32m[Parser::InsertionMode::{:?}] {:?}\x1b[0m",
                self.insertion_mode, token
            );

            let mut token = token.clone();

            // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#tree-construction-dispatcher
            // As each token is emitted from the tokenizer,
            // the user agent must follow the appropriate steps from the following list,
            // known as the tree construction dispatcher:

            // SPEC: If the stack of open elements is empty
            // If the adjusted current node is an element in the HTML namespace
            // FIXME If the adjusted current node is a MathML text integration point and the token is a start tag whose tag name is neither "mglyph" nor "malignmark"
            // FIXME If the adjusted current node is a MathML text integration point and the token is a character token
            // FIXME If the adjusted current node is a MathML annotation-xml element and the token is a start tag whose tag name is "svg"
            // FIXME If the adjusted current node is an HTML integration point and the token is a start tag
            // FIXME If the adjusted current node is an HTML integration point and the token is a character token
            //       If the token is an end-of-file token
            if self.open_elements.is_empty()
                || match &self.adjusted_current_node().unwrap().data {
                    NodeData::Element { name, .. } => name.namespace == Some(Namespace::Html),
                    _ => false,
                }
                || matches!(token, Token::EndOfFile)
            {
                self.process_token(&token);
            } else {
                self.process_token_using_the_rules_for_foreign_content(&mut token)
            }

            self.reprocess_current_token = false;
        }

        self.document
    }
}
