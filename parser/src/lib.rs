use std::rc::Rc;

use dom::custom_element_definition::CustomElementDefinition;
use dom::node::{
    AssociatedValues, Attr, Comment, CustomElementState, DocumentType, Element, Node, NodeType,
    Text,
};
use tokenizer::{Token, Tokenizer};

#[allow(unused)]
#[derive(Debug, Copy, Clone)]
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

enum GenericParsingAlgorithm {
    RawText,
    RCData,
}

pub struct Parser {
    insertion_mode: InsertionMode,
    original_insertion_mode: InsertionMode,
    referenced_insertion_mode: Option<InsertionMode>,
    tokenizer: Tokenizer,
    reprocess_current_token: bool,
    document: Rc<Node>,
    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#stack-of-open-elements
    open_elements: Vec<Rc<Node>>,
    head_element_pointer: Option<Rc<Node>>,
    foster_parenting: bool,
    scripting_flag: bool,
}

const fn is_parser_whitespace(string: char) -> bool {
    if let '\t' | '\u{000a}' | '\u{000c}' | '\u{000d}' | '\u{0020}' = string {
        return true;
    }
    false
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self {
            insertion_mode: InsertionMode::Initial,
            original_insertion_mode: InsertionMode::Initial,
            referenced_insertion_mode: None,
            tokenizer: Tokenizer::new(input),
            reprocess_current_token: false,
            document: Rc::new(Node::new(NodeType::Document {})),
            open_elements: Vec::new(),
            head_element_pointer: None,
            foster_parenting: false,
            scripting_flag: false,
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#current-node
    fn current_node(&self) -> Option<Rc<Node>> {
        // SPEC: The current node is the bottommost node in this stack of open elements.
        self.open_elements.last().cloned()
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#appropriate-place-for-inserting-a-node
    fn appropriate_place_for_inserting_node(
        &self,
        override_target: Option<Rc<Node>>,
    ) -> Option<Rc<Node>> {
        let target = match override_target {
            // SPEC: 1. If there was an override target specified, then let target be the override target.
            Some(override_target) => Some(override_target),
            // SPEC: Otherwise, let target be the current node.
            None => self.current_node(),
        };

        // SPEC: 2. Determine the adjusted insertion location using the first matching steps from the following list:
        if self.foster_parenting {
            // SPEC: If foster parenting is enabled and target is a table, tbody, tfoot, thead, or tr element
            todo!()
        }

        // SPEC: Otherwise, let adjusted insertion location be inside target, after its last child (if any).
        return target;
    }

    fn push_element_to_stack_of_open_elements(&mut self, element: Rc<Node>) {
        self.open_elements.push(element);
    }

    fn pop_current_element_off_stack_of_open_elements(&mut self) {
        self.open_elements.pop();
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#insert-a-character
    fn insert_character(&self, data: char) {
        // SPEC: 2. Let the adjusted insertion location be the appropriate place for inserting a node.
        if let Some(adjusted_insertion_location) = self.appropriate_place_for_inserting_node(None) {
            // SPEC: 3. If the adjusted insertion location is in a Document node, then return.
            if adjusted_insertion_location.is_document() {
                return;
            }

            // SPEC: 4. If there is a Text node immediately before the adjusted insertion location,
            //          then append data to that Text node's data.
            if let Some(last_child) = adjusted_insertion_location.last_child.clone() {
                if let NodeType::Text(mut text) = last_child.node_type.clone() {
                    text.data.push(data)
                }
            } else {
                // SPEC: 5. Otherwise, create a new Text node
                //          whose data is data
                //          and whose node document is the same as that of the element in which the adjusted insertion location finds itself,
                //          and insert the newly created node at the adjusted insertion location.
                let mut text_node = Node::new(NodeType::Text(Text::new(data.to_string().as_str())));
                text_node.document = adjusted_insertion_location.document.clone();
                adjusted_insertion_location.append_child(Rc::new(text_node));
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#insert-a-comment
    fn insert_comment(&self, data: &str, position: Option<Rc<Node>>) {
        let adjusted_insertion_location = match position {
            // SPEC: 2. If position was specified, then let the adjusted insertion location be position.
            Some(position) => Some(position),
            // SPEC:    Otherwise, let adjusted insertion location be the appropriate place for inserting a node.
            None => self.appropriate_place_for_inserting_node(None),
        };
        // SPEC: 3. Create a Comment node whose data attribute is set to data
        //          and whose node document is the same as that of the node in which the adjusted insertion location finds itself.
        let node = Node::new(NodeType::Comment(Comment::new(data)));
        // SPEC: 4. Insert the newly created node at the adjusted insertion location.
        if let Some(adjusted_insertion_location) = adjusted_insertion_location {
            adjusted_insertion_location.append_child(Rc::new(node));
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#insert-a-foreign-element
    fn insert_foreign_element_for_token(
        &mut self,
        token: &Token,
        _namespace: Option<&str>,
    ) -> Rc<Node> {
        // SPEC: 1. Let the adjusted insertion location be the appropriate place for inserting a node.
        let adjusted_insert_location = self.appropriate_place_for_inserting_node(None).unwrap();

        // SPEC: 2. Let element be the result of creating an element for the token in the given namespace,
        //          with the intended parent being the element in which the adjusted insertion location finds itself.
        let parent = adjusted_insert_location.parent_element.clone();
        let element = self.create_element_for_token(token, None, parent).unwrap();
        let element = Rc::new(element);

        // SPEC: 3. If it is possible to insert element at the adjusted insertion location, then:
        // SPEC: 3.1. If the parser was not created as part of the HTML fragment parsing algorithm,
        //            then push a new element queue onto element's relevant agent's custom element reactions stack.
        // FIXME: Implement

        // SPEC: 3.2. Insert element at the adjusted insertion location.
        // FIXME: Implement

        // SPEC: 3.3. If the parser was not created as part of the HTML fragment parsing algorithm,
        //            then pop the element queue from element's relevant agent's custom element reactions stack,
        //            and invoke custom element reactions in that queue.
        // FIXME: Implement

        // SPEC: 4. Push element onto the stack of open elements so that it is the new current node.
        self.push_element_to_stack_of_open_elements(element.clone());

        // SPEC: 5. Return element.
        element
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#insert-an-html-element
    fn insert_html_element_for_token(&mut self, token: &Token) -> Rc<Node> {
        self.insert_foreign_element_for_token(token, None)
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-create-element
    fn create_element(
        &self,
        document: Rc<Node>,
        local_name: &str,
        namespace: Option<&str>,
        prefix: Option<&str>,
        is: Option<&str>,
        _synchronous_custom_element: bool,
    ) -> Option<Node> {
        // SPEC: 3. Let result be null.
        // let mut result = None;

        // SPEC: 4. Let definition be the result of looking up a custom element definition given document, namespace, localName, and is.
        // let definition =
        //     self.look_up_custom_element_definition(document.clone(), namespace, local_name, is);

        // SPEC: 5. If definition is non-null,
        //          and definition's name is not equal to its local name
        //          (i.e., definition represents a customized built-in element), then:
        // FIXME: Implement

        // SPEC: 6. Otherwise, if definition is non-null, then:
        // FIXME: Implement

        // SPEC: 7. Otherwise:
        // SPEC: 7.1. Let interface be the element interface for localName and namespace.
        // SPEC: 7.2. Set result to a new element that implements interface,
        //            with no attributes,
        //            namespace set to namespace,
        //            namespace prefix set to prefix,
        //            local name set to localName,
        //            custom element state set to "uncustomized",
        //            custom element definition set to null,
        //            is value set to is,
        //            and node document set to document.
        let associated_values = AssociatedValues {
            namespace: namespace.map(str::to_string),
            namespace_prefix: prefix.map(str::to_string),
            local_name: local_name.to_string(),
            custom_element_state: CustomElementState::Uncustomized,
            custom_element_definition: None,
            is: is.map(str::to_string),
        };
        let mut node = Node::new(NodeType::Element(Element::new(
            associated_values,
            local_name.to_string(),
            None,
            None,
            String::new(),
        )));
        node.document = Some(document);

        let result = Some(node);

        // SPEC: 7.3. If namespace is the HTML namespace,
        //            and either localName is a valid custom element name or is is non-null,
        //            then set result's custom element state to "undefined".
        // FIXME: Implement

        // SPEC: 8. Return result.
        result
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#create-an-element-for-the-token
    fn create_element_for_token(
        &self,
        token: &Token,
        namespace: Option<&str>,
        _parent: Option<Rc<Node>>,
    ) -> Option<Node> {
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
        let document = self.document.clone();

        // SPEC: 4. Let local name be the tag name of the token.
        // SPEC: 5. Let is be the value of the "is" attribute in the given token,
        //          if such an attribute exists,
        //          or null otherwise.
        let (local_name, is, attributes) = match token {
            Token::StartTag {
                name, attributes, ..
            } => {
                let is = attributes.iter().find_map(|attr| match attr.name.as_str() {
                    "is" => Some(attr.value.as_str()),
                    _ => None,
                });

                (name, is, attributes)
            }
            _ => panic!("Can't create a new element from any other token than a StartTag!"),
        };

        // SPEC: 6. Let definition be the result of looking up a custom element definition
        //          given document, given namespace, local name, and is.
        let _definition =
            self.look_up_custom_element_definition(document.clone(), namespace, local_name, is);

        // SPEC: 7. If definition is non-null and the parser was not created as part of the HTML fragment parsing algorithm,
        //          then let will execute script be true.
        //          Otherwise, let it be false.
        // FIXME: Implement

        // SPEC: 8. If will execute script is true, then:
        //      SPEC: 8.1. Increment document's throw-on-dynamic-markup-insertion counter.
        //      FIXME: Implement

        //      SPEC: 8.2. If the JavaScript execution context stack is empty,
        //                 then perform a microtask checkpoint.
        //      FIXME: Implement

        //      SPEC: 8.3. Push a new element queue onto document's relevant agent's custom element reactions stack.
        //      FIXME: Implement

        // SPEC: 9. Let element be the result of creating an element
        //          given document, localName, given namespace, null, and is.
        //          If will execute script is true,
        //          set the synchronous custom elements flag;
        //          otherwise, leave it unset.
        let mut element_node =
            match self.create_element(document, &local_name, None, None, is, false) {
                Some(element) => element,
                None => return None,
            };

        // SPEC: 10. Append each attribute in the given token to element.
        if let NodeType::Element(element) = &mut element_node.node_type {
            for attr in attributes.iter() {
                element.attributes.insert(
                    attr.name.clone(),
                    NodeType::Attr(Attr::new(attr.value.clone())),
                );
            }
        }
        // SPEC: 11. If will execute script is true, then:
        // SPEC: 11.1. Let queue be the result of popping from document's relevant agent's custom element reactions stack.
        //             (This will be the same element queue as was pushed above.)
        // FIXME: Implement

        // SPEC: 11.2. Invoke custom element reactions in queue.
        // FIXME: Implement

        // SPEC: 11.3. Decrement document's throw-on-dynamic-markup-insertion counter.
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
        Some(element_node)
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/custom-elements.html#look-up-a-custom-element-definition
    fn look_up_custom_element_definition(
        &self,
        _document: Rc<Node>,
        namespace: Option<&str>,
        _local_name: &str,
        _is: Option<&str>,
    ) -> Option<CustomElementDefinition> {
        // SPEC: 1. If namespace is not the HTML namespace, return null.
        if namespace != Some("http://www.w3.org/1999/xhtml") {
            return None;
        }

        // SPEC: 2. If document's browsing context is null, return null.
        // FIXME: Implement

        // SPEC: 3. Let registry be document's relevant global object's CustomElementRegistry object.
        // FIXME: Implement

        // SPEC: 4. If there is custom element definition in registry with name and local name both equal to localName, return that custom element definition.
        // FIXME: Implement

        // SPEC: 5. If there is a custom element definition in registry with name equal to is and local name equal to localName, return that custom element definition.
        // FIXME: Implement

        // SPEC: 6. Return null.
        None
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
            InsertionMode::AfterHead => todo!("InsertionMode::AfterHead"),
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
                let new_data = data.clone();
                self.insert_comment(new_data.as_str(), None);
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
                if name != &Some(String::from("html"))
                    || public_identifier != &None
                    || (system_identifier != &None
                        && system_identifier != &Some(String::from("about:legacy-compat")))
                {
                    // SPEC: then there is a parse error.
                    todo!()
                }

                // SPEC: Append a DocumentType node to the Document node,
                //       with its name set to the name given in the DOCTYPE token, or the empty string if the name was missing;
                //       its public ID set to the public identifier given in the DOCTYPE token, or the empty string if the public identifier was missing;
                //       and its system ID set to the system identifier given in the DOCTYPE token, or the empty string if the system identifier was missing.
                let doctype_node = Node::new(NodeType::DocumentType(DocumentType::new(
                    // FIXME: These clones are quite ugly. Are they needed?
                    name.clone().unwrap_or(String::new()),
                    public_identifier.clone().unwrap_or(String::new()),
                    system_identifier.clone().unwrap_or(String::new()),
                )));
                self.document.append_child(Rc::new(doctype_node));

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
                // FIXME: Do something with the error?
            }
            Token::Comment { .. } => todo!(),
            Token::Character { data } if is_parser_whitespace(*data) => {
                // SPEC: Ignore the token.
            }
            Token::StartTag { name, .. } if name == "html" => {
                // SPEC: Create an element for the token FIXME{in the HTML namespace},
                //       with the Document as the intended parent.
                let token = token.clone();
                let element = Rc::new(
                    self.create_element_for_token(&token, None, Some(self.document.clone()))
                        .unwrap(),
                ); // FIXME: We shouldn't unwrap here. Propogating errors sounds like a good idea here. (or not as the parser might stop?)

                // SPEC: Append it to the Document object.
                self.document.append_child(element.clone());

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
                todo!();
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-before-head-insertion-mode
    fn handle_before_head_insertion_mode(&mut self, token: &Token) {
        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                // SPEC: Ignore the token.
            }
            Token::Comment { .. } => todo!(),
            Token::Doctype { .. } => todo!(),
            Token::StartTag { name, .. } if name == "html" => todo!(),
            Token::StartTag { name, .. } if name == "head" => {
                let token = token.clone();
                self.insert_html_element_for_token(&token);
            }
            Token::EndTag { name, .. } if name == "head" || name == "body" || name == "br" => {
                todo!()
            }
            Token::EndTag { .. } => todo!(),
            _ => {
                // SPEC: Insert an HTML element for a "head" start tag token with no attributes.
                let element = self.insert_html_element_for_token(&Token::StartTag {
                    name: String::from("head"),
                    self_closing: false,
                    attributes: Vec::new(),
                });
                // SPEC: Set the head element pointer to the newly created head element.
                self.head_element_pointer = Some(element);
                // SPEC: Switch the insertion mode to "in head".
                self.switch_insertion_mode_to(InsertionMode::InHead);
                // SPEC: Reprocess the current token.
                self.reprocess_token();
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inhead
    fn handle_in_head_insertion_mode(&mut self, token: &Token) {
        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                // SPEC: Insert the character.
                self.insert_character(*data);
            }
            Token::Comment { data } => {
                // SPEC: Insert a comment.
                self.insert_comment(data, None);
            }
            Token::Doctype { .. } => {
                // SPEC: Parse error. Ignore the token.
            }
            Token::StartTag { name, .. } if name == "html" => {
                // SPEC: Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_rules_of(InsertionMode::InBody)
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
                todo!()
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

    fn handle_text_insertion_mode(&mut self, token: &Token) {
        match token {
            Token::Character { data } => {
                self.insert_character(*data);
            }
            Token::EndOfFile => {
                // SPEC: Parse error.

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

    pub fn parse(&mut self) -> Node {
        while let Some(token) = match self.reprocess_current_token {
            true => self.tokenizer.current_token(),
            false => self.tokenizer.next_token(),
        } {
            eprintln!("[{:?}] {:?}", self.insertion_mode, token);

            let token = token.clone();
            self.process_token(&token);

            self.reprocess_current_token = false;
        }
        self.document.clone().as_ref().to_owned()
    }
}
