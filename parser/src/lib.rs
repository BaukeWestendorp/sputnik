use std::collections::HashMap;
use std::rc::Rc;

use dom::custom_element_definition::CustomElementDefinition;
use dom::node::{
    AssociatedValues, Attr, Comment, CustomElementState, DocumentType, Element, Node, NodeType,
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

pub struct Parser {
    insertion_mode: InsertionMode,
    tokenizer: Tokenizer,
    reprocess_current_token: bool,
    document: Rc<Node>,
    open_elements: Vec<Rc<Node>>,
    head_element_pointer: Option<Rc<Node>>,
    foster_parenting: bool,
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
            tokenizer: Tokenizer::new(input),
            reprocess_current_token: false,
            document: Rc::new(Node::new(NodeType::Document {})),
            open_elements: Vec::new(),
            head_element_pointer: None,
            foster_parenting: false,
        }
    }

    fn current_node(&self) -> Option<Rc<Node>> {
        self.open_elements.last().cloned()
    }

    fn appropriate_place_for_inserting_node(&self) -> Option<Rc<Node>> {
        // SPEC: 1. If there was an override target specified, then let target be the override target.
        // FIXME: Implement
        let target = self.current_node();

        // SPEC: 2. Determine the adjusted insertion location using the first matching steps from the following list:
        if self.foster_parenting {
            // SPEC: If foster parenting is enabled and target is a table, tbody, tfoot, thead, or tr element
            todo!()
        }

        // SPEC: Otherwise, let adjusted insertion location be inside target, after its last child (if any).
        return target;
    }

    fn put_element_in_stack_of_open_elements(&mut self, element: Rc<Node>) {
        self.open_elements.push(element);
    }

    fn insert_comment(&self, data: &str) {
        // SPEC: 2. If position was specified, then let the adjusted insertion location be position.
        // FIXME: Implement
        // SPEC:    Otherwise, let adjusted insertion location be the appropriate place for inserting a node.
        let adjusted_insertion_location = self.appropriate_place_for_inserting_node();
        // SPEC: 3. Create a Comment node whose data attribute is set to data
        //          and whose node document is the same as that of the node in which the adjusted insertion location finds itself.
        let node = Node::new(NodeType::Comment(Comment::new(data)));
        // SPEC: 4. Insert the newly created node at the adjusted insertion location.
        if let Some(adjusted_insertion_location) = adjusted_insertion_location {
            adjusted_insertion_location.append_child(Rc::new(node));
        }
    }

    fn insert_html_element_for_token(&mut self, token: &Token) -> Rc<Node> {
        // SPEC: 1. Let the adjusted insertion location be the appropriate place for inserting a node.
        let adjusted_insert_location = self.appropriate_place_for_inserting_node().unwrap();

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
        self.put_element_in_stack_of_open_elements(element.clone());

        // SPEC: 5. Return element.
        element
    }

    fn look_up_custom_element_definition(
        &self,
        _document: Rc<Node>,
        namespace: Option<&String>,
        _local_name: &String,
        _is: Option<&String>,
    ) -> Option<CustomElementDefinition> {
        // SPEC: 1. If namespace is not the HTML namespace, return null.
        if namespace != Some(&String::from("http://www.w3.org/1999/xhtml")) {
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

    fn create_element(
        &self,
        document: Rc<Node>,
        local_name: &String,
        namespace: Option<&String>,
        prefix: Option<&String>,
        is: Option<&String>,
        _synchronous_custom_element: bool,
    ) -> Option<Node> {
        // SPEC: 3. Let result be null.
        #[allow(unused)]
        let mut result = None;

        // SPEC: 4. Let definition be the result of looking up a custom element definition given document, namespace, localName, and is.
        #[allow(unused)]
        let definition =
            self.look_up_custom_element_definition(document.clone(), namespace, local_name, is);

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
            namespace: namespace.cloned(),
            namespace_prefix: prefix.cloned(),
            local_name: local_name.clone(),
            custom_element_state: CustomElementState::Uncustomized,
            custom_element_definition: None,
            is: is.cloned(),
        };
        let mut node = Node::new(NodeType::Element(Element::new(
            associated_values,
            local_name.clone(),
            None,
            None,
            String::new(),
        )));
        node.owner_document = Some(document);
        result = Some(node);

        // SPEC: 7.3. If namespace is the HTML namespace,
        //            and either localName is a valid custom element name or is is non-null,
        //            then set result's custom element state to "undefined".
        // FIXME: Implement

        // SPEC: 8. Return result.
        result
    }

    fn create_element_for_token(
        &self,
        token: &Token,
        namespace: Option<&String>,
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
                    "is" => Some(&attr.value),
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

    fn switch_insertion_mode(&mut self, insertion_mode: InsertionMode) {
        self.insertion_mode = insertion_mode
    }

    fn reprocess_token(&mut self) {
        self.reprocess_current_token = true;
    }

    pub fn parse(&mut self) -> Node {
        while let Some(token) = match self.reprocess_current_token {
            true => self.tokenizer.current_token(),
            false => self.tokenizer.next_token(),
        } {
            eprintln!("[{:?}] {:?}", self.insertion_mode, token);

            match self.insertion_mode {
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode
                InsertionMode::Initial => match token {
                    Token::Character { data } if is_parser_whitespace(*data) => {
                        // SPEC: Ignore the token.
                    }
                    Token::Comment { data } => {
                        // SPEC: Insert a comment as the last child of the Document object.
                        let new_data = data.clone();
                        self.insert_comment(new_data.as_str());
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
                        self.switch_insertion_mode(InsertionMode::BeforeHtml);
                    }
                    _ => {
                        // SPEC: If the document is not an iframe srcdoc document, then this is a parse error;
                        //       if the parser cannot change the mode flag is false, set the Document to quirks mode.
                        // FIXME: Implement

                        // SPEC: In any case, switch the insertion mode to "before html",
                        //       then reprocess the token.
                        self.switch_insertion_mode(InsertionMode::BeforeHtml);
                        self.reprocess_token();
                    }
                },
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-before-html-insertion-mode
                InsertionMode::BeforeHtml => match token {
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
                            self.create_element_for_token(
                                &token,
                                None,
                                Some(self.document.clone()),
                            )
                            .unwrap(),
                        ); // FIXME: We shouldn't unwrap here. Propogating errors sounds like a good idea here. (or not as the parser might stop?)

                        // SPEC: Append it to the Document object.
                        self.document.append_child(element.clone());

                        // SPEC: Put this element in the stack of open elements.
                        self.put_element_in_stack_of_open_elements(element);
                        // SPEC: Switch the insertion mode to "before head".
                        self.switch_insertion_mode(InsertionMode::BeforeHead);
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
                },
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-before-head-insertion-mode
                InsertionMode::BeforeHead => match token {
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
                    Token::EndTag { name, .. }
                        if name == "head" || name == "body" || name == "br" =>
                    {
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
                        self.switch_insertion_mode(InsertionMode::InHead);
                        // SPEC: Reprocess the current token.
                        self.reprocess_token();
                    }
                },
                InsertionMode::InHead => todo!(),
                InsertionMode::InHeadNoscript => todo!(),
                InsertionMode::AfterHead => todo!(),
                InsertionMode::InBody => todo!(),
                InsertionMode::Text => todo!(),
                InsertionMode::InTable => todo!(),
                InsertionMode::InTableText => todo!(),
                InsertionMode::InCaption => todo!(),
                InsertionMode::InColumnGroup => todo!(),
                InsertionMode::InTableBody => todo!(),
                InsertionMode::InRow => todo!(),
                InsertionMode::InCell => todo!(),
                InsertionMode::InSelect => todo!(),
                InsertionMode::InSelectInTable => todo!(),
                InsertionMode::InTemplate => todo!(),
                InsertionMode::AfterBody => todo!(),
                InsertionMode::InFrameset => todo!(),
                InsertionMode::AfterFrameset => todo!(),
                InsertionMode::AfterAfterBody => todo!(),
                InsertionMode::AfterAfterFrameset => todo!(),
            }
        }
        self.document.clone().as_ref().to_owned()
    }
}