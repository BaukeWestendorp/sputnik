use std::cell::RefCell;

use crate::list_of_active_formatting_elements::{
    ActiveFormattingElement, ListOfActiveFormattingElements,
};
use crate::stack_of_open_elements::StackOfOpenElements;
use dom::arena::{Arena, Ref};
use dom::node::{CharacterDataVariant, Node, NodeData};
use dom::{Namespace, QualifiedName};
use tokenizer::{Token, Tokenizer};

mod list_of_active_formatting_elements;
mod stack_of_open_elements;

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
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
enum InsertionLocation<'arena> {
    BeforeElement(Ref<'arena>),
    AfterLastChildIfAny,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
struct AdjustedInsertionLocation<'arena> {
    parent: Ref<'arena>,
    child: InsertionLocation<'arena>,
}

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
    tokenizer: Tokenizer,
    pending_table_character_tokens: Vec<Token>,
    stack_of_open_elements: StackOfOpenElements<'arena>,
    list_of_active_formatting_elements: ListOfActiveFormattingElements<'arena>,
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
            tokenizer: Tokenizer::new(input),
            pending_table_character_tokens: Vec::new(),
            stack_of_open_elements: StackOfOpenElements::new(),
            list_of_active_formatting_elements: ListOfActiveFormattingElements::new(),
            head_element: None,
            foster_parenting: false,
            scripting_flag: false,
            frameset_ok: FramesetState::Ok,
        }
    }

    fn new_node(&self, document: Ref<'arena>, data: NodeData) -> Ref<'arena> {
        self.arena.alloc(Node::new(Some(document), data))
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#generate-implied-end-tags
    fn generate_implied_end_tags_except_for(&mut self, except_for: &str) {
        // SPEC: while the current node is a dd element, a dt element, an li element, an optgroup element,
        //       an option element, a p element, an rb element, an rp element, an rt element, or an rtc element,
        //       the UA must pop the current node off the stack of open elements.
        let mut current = self.current_node();
        while let Some(node) = current {
            if node.element_tag_name() == Some(except_for) {
                break;
            }

            if node.is_element_with_one_of_tags(&[
                "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc",
            ]) {
                return;
            }
            self.stack_of_open_elements.pop_current_element();
            current = self.current_node();
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#push-onto-the-list-of-active-formatting-elements
    fn push_onto_the_list_of_active_formatting_elements(&self, element: Ref<'arena>) {
        // SPEC: 1. If there are already three elements in the list of active formatting elements after the last marker, if any,
        //          or anywhere in the list if there are no markers, that have the same tag name, namespace, and attributes as element,
        //          then remove the earliest such element from the list of active formatting elements.
        //          For these purposes, the attributes must be compared as they were when the elements were created by the parser;
        //          two elements have the same attributes if all their parsed attributes can be paired such that
        //          the two attributes in each pair have identical names, namespaces, and values (the order of the attributes does not matter).
        todo!()
    }

    fn switch_insertion_mode_to(&mut self, insertion_mode: InsertionMode) {
        self.insertion_mode = insertion_mode
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#reset-the-insertion-mode-appropriately
    fn reset_insertion_mode_appropriately(&mut self) {
        // SPEC: 1. Let last be false.
        let mut last = false;
        // SPEC: 2. Let node be the last node in the stack of open elements.
        let mut node = self.current_node().unwrap();
        // SPEC: 3. Loop:
        loop {
            // If node is the first node in the stack of open elements, then set last to true,
            // FIXME{and, if the parser was created as part of the HTML fragment parsing algorithm (fragment case), set node to the context element passed to that algorithm.}
            if Node::are_same_optional(Some(node), self.stack_of_open_elements.first()) {
                last = true;
            }

            // FIXME: Convert this to a match statement.

            // SPEC: 4. If node is a select element, run these substeps:
            if node.is_element_with_tag("select") {
                // SPEC: 4.1 If last is true, jump to the step below labeled done.
                if !last {
                    // SPEC: 4.2 Let ancestor be node.
                    let mut ancestor = node;
                    // SPEC: 4.3 Loop: If ancestor is the first node in the stack of open elements, jump to the step below labeled done.
                    while !Node::are_same_optional(
                        Some(ancestor),
                        self.stack_of_open_elements.first(),
                    ) {
                        // SPEC: 4.4 Let ancestor be the node before ancestor in the stack of open elements.
                        ancestor = self
                            .stack_of_open_elements
                            .element_immediately_above(ancestor)
                            .expect("element should exist, because, otherwise we would have broken out of this loop");
                        // SPEC: 4.5 If ancestor is a template node, jump to the step below labeled done.
                        if ancestor.is_element_with_tag("template") {
                            break;
                        }
                        // SPEC: 4.6 If ancestor is a table node, switch the insertion mode to "in select in table" and return.
                        if ancestor.is_element_with_tag("table") {
                            self.switch_insertion_mode_to(InsertionMode::InSelectInTable);
                            return;
                        }
                        // SPEC: 4.7 Jump back to the step labeled loop.
                    }
                }
                // SPEC: 4.8 Done: Switch the insertion mode to "in select" and return.
                self.switch_insertion_mode_to(InsertionMode::InSelect);
            }
            // SPEC: 5. If node is a td or th element and last is false, then switch the insertion mode to "in cell" and return.
            if node.is_element_with_one_of_tags(&["td", "tr"]) && !last {
                self.switch_insertion_mode_to(InsertionMode::InCell);
                return;
            }
            // SPEC: 6. If node is a tr element, then switch the insertion mode to "in row" and return.
            if node.is_element_with_tag("tr") {
                self.switch_insertion_mode_to(InsertionMode::InRow);
                return;
            }
            // SPEC: 7. If node is a tbody, thead, or tfoot element, then switch the insertion mode to "in table body" and return.
            if node.is_element_with_one_of_tags(&["tbody", "thead", "tfoot"]) {
                self.switch_insertion_mode_to(InsertionMode::InTableBody);
                return;
            }
            // SPEC: 8. If node is a caption element, then switch the insertion mode to "in caption" and return.
            if node.is_element_with_tag("caption") {
                self.switch_insertion_mode_to(InsertionMode::InCaption);
                return;
            }
            // SPEC: 9. If node is a colgroup element, then switch the insertion mode to "in column group" and return.
            if node.is_element_with_tag("colgroup") {
                self.switch_insertion_mode_to(InsertionMode::InColumnGroup);
                return;
            }
            // SPEC: 10. If node is a table element, then switch the insertion mode to "in table" and return.
            if node.is_element_with_tag("table") {
                self.switch_insertion_mode_to(InsertionMode::InTable);
                return;
            }
            // SPEC: 11. If node is a template element, then switch the insertion mode to the current template insertion mode and return.
            if node.is_element_with_tag("template") {
                todo!();
            }
            // SPEC: 12. If node is a head element and last is false, then switch the insertion mode to "in head" and return.
            if node.is_element_with_tag("head") && !last {
                self.switch_insertion_mode_to(InsertionMode::InHead);
                return;
            }
            // SPEC: 13. If node is a body element, then switch the insertion mode to "in body" and return.
            if node.is_element_with_tag("body") {
                self.switch_insertion_mode_to(InsertionMode::InBody);
                return;
            }
            // SPEC: 14. If node is a frameset element, then switch the insertion mode to "in frameset" and return. (fragment case)
            if node.is_element_with_tag("frameset") {
                self.switch_insertion_mode_to(InsertionMode::InFrameset);
                return;
            }
            // SPEC: 15. If node is an html element, run these substeps:
            if node.is_element_with_tag("html") {
                // SPEC: 15.1 If the head element pointer is null,
                if self.head_element.is_none() {
                    // SPEC: switch the insertion mode to "before head" and return. (fragment case)
                    self.switch_insertion_mode_to(InsertionMode::BeforeHead);
                    return;
                }
                // SPEC: 15.2 Otherwise, the head element pointer is not null, switch the insertion mode to "after head" and return.
                self.switch_insertion_mode_to(InsertionMode::AfterHead);
            }
            // SPEC: 16. If last is true, then switch the insertion mode to "in body" and return. (fragment case)
            if last {
                self.switch_insertion_mode_to(InsertionMode::InBody);
                return;
            }

            // SPEC: 17. Let node now be the node before node in the stack of open elements.
            node = self
                .stack_of_open_elements
                .element_immediately_above(node)
                .unwrap();

            // SPEC: 18. Return to the step labeled loop.
        }
    }

    fn reprocess_token(&mut self, token: &mut Token) {
        self.process_token_using_the_rules_for(self.insertion_mode, token);
    }

    fn process_token_using_the_rules_for(
        &mut self,
        insertion_mode: InsertionMode,
        token: &mut Token,
    ) {
        eprintln!(
            "\x1b[32m[Parser::InsertionMode::{:?}] {:?}\x1b[0m",
            insertion_mode, token
        );

        match insertion_mode {
            InsertionMode::Initial => self.handle_initial(token),
            InsertionMode::BeforeHtml => self.handle_before_html(token),
            InsertionMode::BeforeHead => self.handle_before_head(token),
            InsertionMode::InHead => self.handle_in_head(token),
            InsertionMode::InHeadNoscript => todo!("InsertionMode::InHeadNoscript"),
            InsertionMode::AfterHead => self.handle_after_head(token),
            InsertionMode::InBody => self.handle_in_body(token),
            InsertionMode::Text => self.handle_text(token),
            InsertionMode::InTable => self.handle_in_table(token),
            InsertionMode::InTableText => self.handle_in_table_text(token),
            InsertionMode::InCaption => todo!("InsertionMode::InCaption"),
            InsertionMode::InColumnGroup => todo!("InsertionMode::InColumnGroup"),
            InsertionMode::InTableBody => self.handle_in_table_body(token),
            InsertionMode::InRow => self.handle_in_row(token),
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

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#current-node
    fn current_node(&self) -> Option<Ref<'arena>> {
        // SPEC: The current node is the bottommost node in this stack of open elements.
        self.stack_of_open_elements.current_node()
    }

    fn current_node_is_one_of_elements_with_tag(&self, elements: &[&str]) -> bool {
        if let Some(current_node) = self.current_node() {
            return current_node.is_element_with_one_of_tags(elements);
        }
        false
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
    ) -> AdjustedInsertionLocation<'arena> {
        let target = match override_target {
            // SPEC: 1. If there was an override target specified, then let target be the override target.
            Some(override_target) => override_target,
            // SPEC: Otherwise, let target be the current node.
            None => self
                .current_node()
                .expect("There will always be an html element on the stack"),
        };

        // SPEC: 2. Determine the adjusted insertion location using the first matching steps from the following list:
        // SPEC:    -> If foster parenting is enabled and target is a table, tbody, tfoot, thead, or tr element
        return if self.foster_parenting
            && target.is_element_with_one_of_tags(&["table", "tbody", "tfoot", "thead", "tr"])
        {
            // SPEC: 2.1 Let last template be the last template element in the stack of open elements, if any.
            let last_template = self.stack_of_open_elements.last_with_tag("template");
            // SPEC: 2.2 Let last table be the last table element in the stack of open elements, if any.
            let last_table = &self.stack_of_open_elements.last_with_tag("table");

            // SPEC: 2.3 If there is a last template and
            //           either there is no last table, or there is one, but last template is lower (more recently added) than last table in the stack of open elements,
            if let Some(last_template) = last_template {
                if last_table.is_none() || last_template.0 > last_table.unwrap().0 {
                    // SPEC: then: let adjusted insertion location be inside last template's template contents, after its last child (if any), and abort these steps.
                    return AdjustedInsertionLocation {
                        parent: last_template.1,
                        child: InsertionLocation::AfterLastChildIfAny,
                    };
                }
            }

            match last_table {
                None => {
                    // SPEC: 2.4 If there is no last table,
                    //           then let adjusted insertion location be inside the first element in the stack of open elements (the html element),
                    //           after its last child (if any),
                    //           and abort these steps. (fragment case)
                    AdjustedInsertionLocation {
                        parent: self.stack_of_open_elements.first().unwrap(),
                        child: InsertionLocation::AfterLastChildIfAny,
                    }
                }
                Some(last_table) => {
                    // SPEC: 2.5 If last table has a parent node, then let adjusted insertion location be inside last table's parent node, immediately before last table, and abort these steps.
                    if let Some(parent) = last_table.1.parent() {
                        return AdjustedInsertionLocation {
                            parent,
                            child: InsertionLocation::BeforeElement(last_table.1),
                        };
                    }

                    // SPEC: 2.6 Let previous element be the element immediately above last table in the stack of open elements.
                    let previous_element = self
                        .stack_of_open_elements
                        .element_immediately_above(last_table.1)
                        .expect("There will always be an html element on the stack");

                    // SPEC: 2.7 Let adjusted insertion location be inside previous element, after its last child (if any).
                    AdjustedInsertionLocation {
                        parent: previous_element,
                        child: InsertionLocation::AfterLastChildIfAny,
                    }
                }
            }
        } else {
            // SPEC: -> Otherwise, let adjusted insertion location be inside target, after its last child (if any).
            AdjustedInsertionLocation {
                parent: target,
                child: InsertionLocation::AfterLastChildIfAny,
            }
        };

        // SPEC: If the adjusted insertion location is inside a template element,
        //       let it instead be inside the template element's template contents, after its last child (if any).
        // FIXME: Implement
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-create-element
    fn create_element(
        &mut self,
        document: Ref<'arena>,
        name: QualifiedName,
        attributes: Vec<dom::Attribute>,
    ) -> Ref<'arena> {
        // FIXME: This does not implement any spec functionality yet!

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

        if adjusted_insertion_location.parent.is_document() {
            return None;
        }

        if let Some(text_node) = adjusted_insertion_location.parent.last_child() {
            return Some(text_node);
        }

        let new_text_node = self.new_node(
            self.document,
            NodeData::CharacterData {
                variant: CharacterDataVariant::Text,
                data: RefCell::new(String::new()),
            },
        );

        adjusted_insertion_location.parent.append(new_text_node);

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
        self.stack_of_open_elements
            .pop_elements_until_element_has_been_popped("p");
    }

    // SPECLINK: https://html.spec.whatwg.org/#insert-a-character
    fn insert_character(&mut self, data: char) {
        if let Some(NodeData::CharacterData {
            data: text_data, ..
        }) = self.find_character_insertion_node().map(|node| &node.data)
        {
            text_data.borrow_mut().push(data);
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
        // eprintln!("{:#?}", adjusted_insertion_location);

        // SPEC: 2. Let element be the result of creating an element for the token in the given namespace,
        //          with the intended parent being the element in which the adjusted insertion location finds itself.
        let element = self.create_element_for_token(token, adjusted_insertion_location.parent);

        let child = match adjusted_insertion_location.child {
            InsertionLocation::BeforeElement(element) => Some(element),
            InsertionLocation::AfterLastChildIfAny => None,
        };

        let pre_insertion_validity = adjusted_insertion_location
            .parent
            .ensure_pre_insertion_validity(element, child);

        // SPEC: 3. If it is possible to insert element at the adjusted insertion location, then:
        if pre_insertion_validity.is_ok() {
            // SPEC: 3.1. If the parser was not created as part of the HTML fragment parsing algorithm,
            //            then push a new element queue onto element's relevant agent's custom element reactions stack.

            // SPEC: 3.2. Insert element at the adjusted insertion location.
            adjusted_insertion_location
                .parent
                .insert_before(element, child);

            // SPEC: 3.3. If the parser was not created as part of the HTML fragment parsing algorithm,
            //            then pop the element queue from element's relevant agent's custom element reactions stack,
            //            and invoke custom element reactions in that queue.
            // FIXME: Implement
        } else {
            eprintln!("FIXME: Throw {:?}", pre_insertion_validity);
        }

        // SPEC: 4. Push element onto the stack of open elements so that it is the new current node.
        self.stack_of_open_elements.push(element);

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
    fn handle_initial(&mut self, token: &mut Token) {
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
                self.reprocess_token(token);
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-before-html-insertion-mode
    fn handle_before_html(&mut self, token: &mut Token) {
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
                self.stack_of_open_elements.push(element);

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
                self.stack_of_open_elements.push(element);

                // SPEC: Switch the insertion mode to "before head", then reprocess the token.
                self.switch_insertion_mode_to(InsertionMode::BeforeHead);
                self.reprocess_token(token);
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-before-head-insertion-mode
    fn handle_before_head(&mut self, token: &mut Token) {
        macro_rules! anything_else {
            () => {
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
                self.reprocess_token(token);
            };
        }

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
                anything_else!();
            }
            Token::EndTag { name, .. } => {
                // SPEC: Parse error. Ignore the token.
                log_parser_error!(format!("Invalid End Tag: {name}"));
            }
            _ => {
                anything_else!();
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inhead
    fn handle_in_head(&mut self, token: &mut Token) {
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
                self.process_token_using_the_rules_for(InsertionMode::InBody, token)
            }
            Token::StartTag { name, .. }
                if name == "base" || name == "basefont" || name == "bgsound" || name == "link" =>
            {
                // SPEC: Insert an HTML element for the token.
                self.insert_html_element_for_token(token);
                // SPEC:Immediately pop the current node off the stack of open elements.
                self.stack_of_open_elements.pop_current_element();
                // SPEC: Acknowledge the token's self-closing flag, if it is set.
                token.acknowledge_self_closing_flag_if_set();
            }
            Token::StartTag { name, .. } if name == "meta" => {
                // SPEC: Insert an HTML element for the token.
                self.insert_html_element_for_token(token);
                // SPEC: Immediately pop the current node off the stack of open elements.
                self.stack_of_open_elements.pop_current_element();
                // SPEC: Acknowledge the token's self-closing flag, if it is set.
                token.acknowledge_self_closing_flag_if_set();
                // SPEC: If the active speculative HTML parser is null, then:
                // FIXME: Implement
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
                self.stack_of_open_elements.pop_current_element();

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
                self.stack_of_open_elements.pop_current_element();

                // SPEC: Switch the insertion mode to "after head".
                self.switch_insertion_mode_to(InsertionMode::AfterHead);

                // SPEC: Reprocess the token.
                self.process_token_using_the_rules_for(self.insertion_mode, token);
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-after-head-insertion-mode
    fn handle_after_head(&mut self, token: &mut Token) {
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
                self.process_token_using_the_rules_for(InsertionMode::InBody, token);
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
                    self.stack_of_open_elements.push(head_element_pointer);
                }

                // SPEC: Process the token using the rules for the "in head" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InHead, token);

                // SPEC: Remove the node pointed to by the head element pointer from the stack of open elements.
                //       (It might not be the current node at this point.)
                if let Some(head_element_pointer) = self.head_element {
                    self.stack_of_open_elements
                        .remove_element(head_element_pointer);
                }
            }
            Token::EndTag { name, .. } if name == "template" => {
                // SPEC: Process the token using the rules for the "in head" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InHead, token);
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
                self.process_token_using_the_rules_for(self.insertion_mode, token);
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody
    fn handle_in_body(&mut self, token: &mut Token) {
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
            Token::StartTag { name, .. } if name == "body" => todo!(),
            Token::StartTag { name, .. } if name == "frameset" => todo!(),
            Token::EndOfFile => {
                // SPEC: If the stack of template insertion modes is not empty, then process the token using the rules for the "in template" insertion mode.
                // FIXME: Implement

                // Otherwise, follow these steps:

                // If there is a node in the stack of open elements that is not either a dd element, a dt element, an li element, an optgroup element, an option element, a p element, an rb element, an rp element, an rt element, an rtc element, a tbody element, a td element, a tfoot element, a th element, a thead element, a tr element, the body element, or the html element, then this is a parse error.
                if !self
                    .stack_of_open_elements
                    .contains_one_of_tags(SPECIAL_TAGS)
                {
                    log_parser_error!();
                };

                // Stop parsing.
                self.stop_parsing();
            }
            Token::EndTag { name, .. } if name == "body" => {
                // SPEC: If the stack of open elements does not have a body element in scope, this is a parse error; ignore the token.
                if !self
                    .stack_of_open_elements
                    .has_element_with_tag_name_in_scope("body")
                {
                    log_parser_error!();
                    return;
                }

                // SPEC: Otherwise, if there is a node in the stack of open elements that is not either a
                //       dd element, a dt element, an li element, an optgroup element, an option element, a p element, an rb element,
                //       an rp element, an rt element, an rtc element, a tbody element, a td element, a tfoot element, a th element,
                //       a thead element, a tr element, the body element, or the html element, then this is a parse error.
                if !self
                    .stack_of_open_elements
                    .contains_one_of_tags(SPECIAL_TAGS)
                {
                    log_parser_error!();
                }

                // SPEC: Switch the insertion mode to "after body".
                self.switch_insertion_mode_to(InsertionMode::AfterBody);
            }
            Token::EndTag { name, .. } if name == "html" => {
                // SPEC: 1. If the stack of open elements does not have a body element in scope,
                //          this is a parse error; ignore the token.
                if !self
                    .stack_of_open_elements
                    .has_element_with_tag_name_in_scope("body")
                {
                    log_parser_error!();
                    return;
                }
                // SPEC: 2. Otherwise, if there is a node in the stack of open elements that is not either a dd element, a dt element, an li element, an optgroup element, an option element, a p element, an rb element, an rp element, an rt element, an rtc element, a tbody element, a td element, a tfoot element, a th element, a thead element, a tr element, the body element, or the html element, then this is a parse error.

                // SPEC: 3. Switch the insertion mode to "after body".
                self.switch_insertion_mode_to(InsertionMode::AfterBody);
                // SPEC: 4. Reprocess the token.
                self.reprocess_token(token);
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
                if self
                    .stack_of_open_elements
                    .has_element_with_tag_name_in_button_scope("p")
                {
                    self.close_a_p_element();
                }
                // SPEC: Insert an HTML element for the token.
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
            Token::StartTag { name, .. } if name == "form" => todo!(),
            Token::StartTag { name, .. } if name == "li" => todo!(),
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
                todo!()
            }
            Token::EndTag { name, .. } if name == "form" => todo!(),
            Token::EndTag { name, .. } if name == "p" => {
                // SPEC: If the stack of open elements does not have a p element in button scope, then this is a parse error;
                //       insert an HTML element for a "p" start tag token with no attributes.
                if !self
                    .stack_of_open_elements
                    .has_element_with_tag_name_in_button_scope("p")
                {
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
            Token::EndTag { name, .. } if name == "li" => {
                todo!()
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
            Token::EndTag { name, .. } if name == "sarcasm" => todo!(),
            Token::StartTag { name, .. } if name == "a" => {
                use list_of_active_formatting_elements::Position;

                // SPEC: If the list of active formatting elements contains an a element between
                //       the end of the list and the last marker on the list (or the start of the list if there is no marker on the list),
                //       then this is a parse error;
                //       run the adoption agency algorithm for the token,
                //       then remove that element from the list of active formatting elements and the stack of open elements
                //       if the adoption agency algorithm didn't already remove it (it might not have if the element is not in table scope).
                if self
                    .list_of_active_formatting_elements
                    .contains_element_between(Position::End, Position::LastMarkerOrElseStart, "a")
                {
                    log_parser_error!();
                    todo!()
                }
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
                // SPEC: Reconstruct the active formatting elements, if any.
                self.reconstruct_active_formatting_elements_if_any();
                // SPEC: Insert an HTML element for the token.
                let element = self.insert_html_element_for_token(token);
                // SPEC: Push onto the list of active formatting elements that element.
                self.push_onto_the_list_of_active_formatting_elements(element);
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
                self.run_the_adoption_agency_algorithm_for_token(token);
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
                // SPEC: If the Document is not set to quirks mode, and the stack of open elements has a p element in button scope, then close a p element.
                // FIXME: Implement
                // SPEC: Insert an HTML element for the token.
                self.insert_html_element_for_token(token);
                // SPEC: Set the frameset-ok flag to "not ok".
                self.frameset_ok = FramesetState::NotOk;
                // SPEC: Switch the insertion mode to "in table".
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
                // SPEC: Reconstruct the active formatting elements, if any.
                self.reconstruct_active_formatting_elements_if_any();
                // SPEC: Insert an HTML element for the token.
                self.insert_html_element_for_token(token);
                // SPEC: Immediately pop the current node off the stack of open elements.
                self.stack_of_open_elements.pop_current_element();
                // SPEC: Acknowledge the token's self-closing flag, if it is set.
                token.acknowledge_self_closing_flag_if_set();
                // SPEC: Set the frameset-ok flag to "not ok".
                self.frameset_ok = FramesetState::NotOk;
            }
            Token::StartTag { name, .. } if name == "input" => todo!(),
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
            Token::StartTag { name, .. } if name == "noscript" && self.scripting_flag => todo!(),
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
                // SPEC: Parser error. Ignore the token.
                log_parser_error!();
            }
            Token::StartTag { .. } => {
                // SPEC: Reconstruct the active formatting elements, if any.
                self.reconstruct_active_formatting_elements_if_any();
                // SPEC: Insert an HTML element for the token.
                self.insert_html_element_for_token(token);
            }
            Token::EndTag { .. } => self.in_body_any_other_end_tag(token),
        }
    }

    fn in_body_any_other_end_tag(&mut self, token: &Token) {
        // SPEC: 1. Initialize node to be the current node (the bottommost node of the stack).
        for node in self.stack_of_open_elements.elements.clone().iter().rev() {
            // SPEC: 2. Loop: If node is an HTML element with the same tag name as the token, then:
            let token_tag_name = token.tag_name().expect("token should be EndTag");
            if node.is_element_with_tag(&token_tag_name) {
                // SPEC: 2.1. Generate implied end tags, except for HTML elements with the same tag name as the token.
                self.generate_implied_end_tags_except_for(&token_tag_name);
                // SPEC: 2.2. If node is not the current node, then this is a parse error.
                if Node::are_same(*node, self.current_node().unwrap()) {
                    log_parser_error!();
                }
                // SPEC: 2.3. Pop all the nodes from the current node up to node, including node,
                while Node::are_same(node, self.current_node().unwrap()) {
                    self.stack_of_open_elements.pop_current_element();
                }
                // SPEC: then stop these steps.
                break;
            } else {
                // SPEC: 3. Otherwise, if node is in the special category,
                if node.is_special_tag() {
                    // SPEC: then this is a parse error; ignore the token,
                    log_parser_error!();
                    // SPEC: and return.
                    return;
                }

                // SPEC: 4. Set node to the previous entry in the stack of open elements.
                // SPEC: 5 Return to the step labeled loop.
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#adoption-agency-algorithm
    fn run_the_adoption_agency_algorithm_for_token(&mut self, token: &Token) {
        // SPEC: 1. Let subject be token's tag name.
        let subject = token.tag_name().expect("token should be EndTag");

        // SPEC: 2. If the current node is an HTML element whose tag name is subject,
        //          and the current node is not in the list of active formatting elements,
        //          then pop the current node off the stack of open elements and return.
        if self.current_node_is_one_of_elements_with_tag(&[&subject])
            && self
                .list_of_active_formatting_elements
                .contains(ActiveFormattingElement::Element(
                    self.current_node().unwrap(),
                ))
        {
            self.stack_of_open_elements.pop_current_element();
            return;
        }

        // SPEC: 3. Let outer loop counter be 0.
        let mut outer_loop_counter = 0;

        // SPEC: 4. While true:
        loop {
            // SPEC: 4.1 If outer loop counter is greater than or equal to 8, then return.
            if outer_loop_counter >= 8 {
                return;
            }

            // SPEC: 4.2 Increment outer loop counter by 1.
            outer_loop_counter += 1;
            // SPEC: 4.3 Let formatting element be the last element in the list of active formatting elements that:
            //     * is between the end of the list and the last marker in the list, if any, or the start of the list otherwise, and
            //     * has the tag name subject.
            let formatting_element = self
                .list_of_active_formatting_elements
                .last_element_that_is_between_index_and_has_tag_name(
                    self.list_of_active_formatting_elements
                        .last_marker_index()
                        .unwrap_or(0),
                    self.list_of_active_formatting_elements.len(),
                    &*subject,
                );
            // SPEC: If there is no such element, then return and instead act as described in the "any other end tag" entry above.
            if formatting_element.is_none() {
                self.in_body_any_other_end_tag(token);
                return;
            }
            let formatting_element = match formatting_element.unwrap() {
                ActiveFormattingElement::Marker => panic!("Should not be a marker!"),
                ActiveFormattingElement::Element(element) => element,
            };

            // SPEC: 4.4 If formatting element is not in the stack of open elements,
            if self.stack_of_open_elements.contains(formatting_element) {
                // SPEC: then this is a parse error;
                log_parser_error!();
                // SPEC: remove the element from the list
                self.list_of_active_formatting_elements
                    .remove(formatting_element);
                // SPEC: and return.
                return;
            }

            todo!()
            // SPEC: 4.5 If formatting element is in the stack of open elements, but the element is not in scope, then this is a parse error; return.
            // SPEC: 4.6 If formatting element is not the current node, this is a parse error. (But do not return.)
            // SPEC: 4.7 Let furthest block be the topmost node in the stack of open elements that is lower in the stack than formatting element, and is an element in the special category. There might not be one.
            // SPEC: 4.8 If there is no furthest block, then the UA must first pop all the nodes from the bottom of the stack of open elements, from the current node up to and including formatting element, then remove formatting element from the list of active formatting elements, and finally return.
            // SPEC: 4.9 Let common ancestor be the element immediately above formatting element in the stack of open elements.
            // SPEC: 4.10 Let a bookmark note the position of formatting element in the list of active formatting elements relative to the elements on either side of it in the list.
            // SPEC: 4.11 Let node and last node be furthest block.
            // SPEC: 4.12 Let inner loop counter be 0.
            // SPEC: 4.13 While true:
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incdata
    fn handle_text(&mut self, token: &mut Token) {
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
                self.stack_of_open_elements.pop_current_element();

                // SPEC: Switch the insertion mode to the original insertion mode and reprocess the token.
                self.switch_insertion_mode_to(self.original_insertion_mode);
                self.reprocess_token(token);
            }
            Token::EndTag { name, .. } if name == "script" => todo!(),
            _ => {
                // SPEC: Pop the current node off the stack of open elements.
                self.stack_of_open_elements.pop_current_element();

                // SPEC: Switch the insertion mode to the original insertion mode.
                self.switch_insertion_mode_to(self.original_insertion_mode);
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intable
    fn handle_in_table(&mut self, token: &mut Token) {
        match token {
            Token::Character { .. }
                if self.current_node_is_one_of_elements_with_tag(&[
                    "table", "tbody", "template", "tfoot", "thead", "tr",
                ]) =>
            {
                // SPEC: Let the pending table character tokens be an empty list of tokens.
                self.pending_table_character_tokens = Vec::new();
                // SPEC: Let the original insertion mode be the current insertion mode.
                self.original_insertion_mode = self.insertion_mode;
                // SPEC: Switch the insertion mode to "in table text" and reprocess the token.
                self.switch_insertion_mode_to(InsertionMode::InTableText);
                self.reprocess_token(token);
            }
            Token::Comment { .. } => todo!(),
            Token::Doctype { .. } => todo!(),
            Token::StartTag { name, .. } if name == "caption" => todo!(),
            Token::StartTag { name, .. } if name == "colgroup" => todo!(),
            Token::StartTag { name, .. } if name == "col" => todo!(),
            Token::StartTag { name, .. }
                if name == "tbody" || name == "tfoot" || name == "thead" =>
            {
                // SPEC: Clear the stack back to a table context. (See below.)
                self.clear_the_stack_back_to_a_table_context();
                // SPEC: Insert an HTML element for the token,
                self.insert_html_element_for_token(token);
                // SPEC: then switch the insertion mode to "in table body".
                self.switch_insertion_mode_to(InsertionMode::InTableBody);
            }
            Token::StartTag { name, .. } if name == "td" || name == "th" || name == "tr" => {
                // SPEC: Clear the stack back to a table context. (See below.)
                self.clear_the_stack_back_to_a_table_context();
                // SPEC: Insert an HTML element for a "tbody" start tag token with no attributes,
                self.insert_html_element_for_token(&Token::StartTag {
                    name: "tbody".to_string(),
                    self_closing: false,
                    self_closing_acknowledged: false,
                    attributes: Vec::new(),
                });
                // SPEC: then switch the insertion mode to "in table body".
                self.switch_insertion_mode_to(InsertionMode::InTableBody);
                // SPEC: Reprocess the current token.
                self.reprocess_token(token);
            }
            Token::StartTag { name, .. } if name == "table" => {
                // SPEC: Parse error.
                log_parser_error!("Invalid token: table in table");
                // SPEC: If the stack of open elements does not have a table element in table scope,
                if !self
                    .stack_of_open_elements
                    .has_element_with_tag_name_in_table_scope("table")
                {
                    // SPEC: ignore the token.
                    return;
                }
                // SPEC: Otherwise:
                // SPEC: Pop elements from this stack until a table element has been popped from the stack.
                self.stack_of_open_elements
                    .pop_elements_until_element_has_been_popped("table");
                // SPEC: Reset the insertion mode appropriately.
                self.reset_insertion_mode_appropriately();
                // SPEC: Reprocess the token.
                self.reprocess_token(token);
            }
            Token::EndTag { name, .. } if name == "table" => {
                // SPEC: If the stack of open elements does not have a table element in table scope,
                if !self
                    .stack_of_open_elements
                    .has_element_with_tag_name_in_table_scope("table")
                {
                    // SPEC: this is a parse error; ignore the token.
                    log_parser_error!();
                    return;
                }

                // SPEC: Otherwise:
                // SPEC: Pop elements from this stack until a table element has been popped from the stack.
                self.stack_of_open_elements
                    .pop_elements_until_element_has_been_popped("table");
                // SPEC: Reset the insertion mode appropriately.
                self.reset_insertion_mode_appropriately();
            }
            Token::EndTag { name, .. }
                if name == "body"
                    || name == "caption"
                    || name == "col"
                    || name == "colgroup"
                    || name == "html"
                    || name == "tbody"
                    || name == "td"
                    || name == "tfoot"
                    || name == "th"
                    || name == "thead"
                    || name == "tr" =>
            {
                // SPEC: Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::StartTag { name, .. }
                if name == "style" || name == "script" || name == "template" =>
            {
                todo!()
            }
            Token::EndTag { name, .. } if name == "template" => todo!(),
            Token::StartTag { name, .. } if name == "input" => todo!(),
            Token::StartTag { name, .. } if name == "form" => todo!(),
            Token::EndOfFile => {
                // SPEC: Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody, token);
            }
            _ => self.in_table_anything_else(token),
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/#clear-the-stack-back-to-a-table-context
    fn clear_the_stack_back_to_a_table_context(&mut self) {
        // SPEC: When the steps above require the UA to clear the stack back to a table context,
        //       it means that the UA must, while the current node is not a table, template, or html element,
        //       pop elements from the stack of open elements.
        while !self.current_node_is_one_of_elements_with_tag(&["table", "template", "html"]) {
            self.stack_of_open_elements.pop_current_element()
        }
    }

    fn in_table_anything_else(&mut self, token: &mut Token) {
        // SPEC: Parse error.
        log_parser_error!(
            "Invalid element in table. Attempting recovery using foster parenting..."
        );
        // SPEC: Enable foster parenting,
        self.foster_parenting = true;
        // SPEC: process the token using the rules for the "in body" insertion mode,
        self.process_token_using_the_rules_for(InsertionMode::InBody, token);
        // SPEC: and then disable foster parenting.
        self.foster_parenting = false;
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intabletext
    fn handle_in_table_text(&mut self, token: &mut Token) {
        match token {
            Token::Character { data } if data == &'\u{0000}' => {
                // SPEC: Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::Character { data } => {
                self.pending_table_character_tokens
                    .push(Token::Character { data: *data });
            }
            _ => {
                // SPEC: If any of the tokens in the pending table character tokens list are character tokens that are not ASCII whitespace,
                let all_are_whitespace = self.pending_table_character_tokens.iter().all(|c| {
                    if let Token::Character { data } = c {
                        return data.is_ascii_whitespace();
                    }
                    false
                });
                if !all_are_whitespace {
                    // SPEC: then this is a parse error:
                    log_parser_error!("Not all pending table character tokens are whitespace");
                    // SPEC: reprocess the character tokens in the pending table character tokens list
                    //       using the rules given in the "anything else" entry in the "in table" insertion mode.
                    for character_token in self.pending_table_character_tokens.clone().iter_mut() {
                        self.in_table_anything_else(character_token)
                    }
                }

                // SPEC: Otherwise, insert the characters given by the pending table character tokens list.
                for pending in self.pending_table_character_tokens.clone().iter() {
                    if let Token::Character { data } = pending {
                        self.insert_character(*data);
                    }
                }

                // SPEC: Switch the insertion mode to the original insertion mode and reprocess the token.
                self.switch_insertion_mode_to(self.original_insertion_mode);
                self.reprocess_token(token);
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/#parsing-main-intbody
    fn handle_in_table_body(&mut self, token: &mut Token) {
        macro_rules! start_tags_and_end_tag {
            () => {
                // SPEC: If the stack of open elements does not have a tbody, thead, or tfoot element in table scope,
                if self
                    .stack_of_open_elements
                    .has_element_with_tag_name_in_table_scope("tbody")
                    || self
                        .stack_of_open_elements
                        .has_element_with_tag_name_in_table_scope("thead")
                    || self
                        .stack_of_open_elements
                        .has_element_with_tag_name_in_table_scope("tfoot")
                {
                    // SPEC: this is a parse error; ignore the token.
                    log_parser_error!();
                }
                // SPEC: Otherwise:
                // SPEC: Clear the stack back to a table body context. (See below.)
                self.clear_the_stack_back_to_a_table_body_context();
                // SPEC: Pop the current node from the stack of open elements.
                self.stack_of_open_elements.pop_current_element();
                // SPEC: Switch the insertion mode to "in table".
                self.switch_insertion_mode_to(InsertionMode::InTable);
                // SPEC: Reprocess the token.
                self.reprocess_token(token);
            };
        }

        match token {
            Token::StartTag { name, .. } if name == "tr" => {
                // SPEC: Clear the stack back to a table body context. (See below.)
                self.clear_the_stack_back_to_a_table_body_context();
                // SPEC: Insert an HTML element for the token,
                self.insert_html_element_for_token(token);
                // SPEC: then switch the insertion mode to "in row".
                self.switch_insertion_mode_to(InsertionMode::InRow);
            }
            Token::StartTag { name, .. } if name == "th" || name == "td" => {
                // SPEC: Parse error.
                log_parser_error!();
                // SPEC: Clear the stack back to a table body context. (See below.)
                self.clear_the_stack_back_to_a_table_body_context();
                // SPEC: Insert an HTML element for a "tr" start tag token with no attributes,
                self.insert_html_element_for_token(&Token::StartTag {
                    name: "tr".to_string(),
                    self_closing: false,
                    self_closing_acknowledged: false,
                    attributes: vec![],
                });
                // SPEC: then switch the insertion mode to "in row".
                self.switch_insertion_mode_to(InsertionMode::InRow);
                // SPEC: Reprocess the current token.
                self.reprocess_token(token);
            }
            Token::EndTag { name, .. } if name == "tbody" || name == "tfoot" || name == "thead" => {
                // SPEC: If the stack of open elements does not have an element in table scope that
                //       is an HTML element with the same tag name as the token,
                if let Some(token_tag_name) = token.tag_name() {
                    if self
                        .stack_of_open_elements
                        .has_element_with_tag_name_in_table_scope(&token_tag_name)
                    {
                        // SPEC: this is a parse error; ignore the token.
                        log_parser_error!();
                        return;
                    }
                }
                // SPEC: Otherwise:
                // SPEC: Clear the stack back to a table body context. (See below.)
                self.clear_the_stack_back_to_a_table_body_context();
                // SPEC: Pop the current node from the stack of open elements.
                self.stack_of_open_elements.pop_current_element();
                // SPEC: Switch the insertion mode to "in table".
                self.switch_insertion_mode_to(InsertionMode::InTable);
            }
            Token::StartTag { name, .. }
                if name == "caption"
                    || name == "col"
                    || name == "colgroup"
                    || name == "tbody"
                    || name == "tfoot"
                    || name == "thead" =>
            {
                start_tags_and_end_tag!();
            }
            Token::EndTag { name, .. } if name == "table" => {
                start_tags_and_end_tag!();
            }
            Token::EndTag { name, .. }
                if name == "body"
                    || name == "caption"
                    || name == "col"
                    || name == "colgroup"
                    || name == "html"
                    || name == "td"
                    || name == "th"
                    || name == "body" =>
            {
                // SPEC: Parse error. Ignore the token.
                log_parser_error!();
            }
            _ => {
                // SPEC: Process the token using the rules for the "in table" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InTable, token);
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/#clear-the-stack-back-to-a-table-body-context
    fn clear_the_stack_back_to_a_table_body_context(&mut self) {
        // SPEC: When the steps above require the UA to clear the stack back to a table body context,
        //       it means that the UA must, while the current node is not a tbody, tfoot, thead, template, or html element,
        //       pop elements from the stack of open elements.
        while !self.current_node_is_one_of_elements_with_tag(&[
            "tbody", "tfoot", "thead", "template", "html",
        ]) {
            self.stack_of_open_elements.pop_current_element();
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/#parsing-main-intr
    fn handle_in_row(&mut self, token: &mut Token) {
        match token {
            Token::StartTag { name, .. } if name == "th" || name == "td" => {
                // SPEC: Clear the stack back to a table row context. (See below.)
                self.clear_the_stack_back_to_a_table_row_context();
                // SPEC: Insert an HTML element for the token, then switch the insertion mode to "in cell".
                self.insert_html_element_for_token(&token);
                // SPEC: Insert a marker at the end of the list of active formatting elements.
                self.list_of_active_formatting_elements
                    .insert_marker_at_end();
            }
            Token::EndTag { name, .. } if name == "tr" => {
                // SPEC: If the stack of open elements does not have a tr element in table scope,
                if !self
                    .stack_of_open_elements
                    .has_element_with_tag_name_in_table_scope("tr")
                {
                    // SPEC: this is a parse error; ignore the token.
                }

                // SPEC: Otherwise:
                // SPEC: Clear the stack back to a table row context. (See below.)
                self.clear_the_stack_back_to_a_table_row_context();
                // SPEC: Pop the current node (which will be a tr element) from the stack of open elements.
                self.stack_of_open_elements.pop_current_element();
                // SPEC: Switch the insertion mode to "in table body".
                self.switch_insertion_mode_to(InsertionMode::InTableBody);
            }
            Token::StartTag { name, .. }
                if name == "caption"
                    || name == "col"
                    || name == "colgroup"
                    || name == "tbody"
                    || name == "tfoot"
                    || name == "thead"
                    || name == "tr" =>
            {
                todo!()
            }
            Token::EndTag { name, .. } if name == "table" => {
                todo!()
            }
            Token::EndTag { name, .. } if name == "tbody" || name == "tfoot" || name == "thead" => {
                todo!()
            }
            Token::EndTag { name, .. }
                if name == "body"
                    || name == "caption"
                    || name == "col"
                    || name == "colgroup"
                    || name == "html"
                    || name == "td"
                    || name == "th" =>
            {
                // SPEC: Parse error. Ignore the token.
            }
            _ => {
                // SPEC: Process the token using the rules for the "in table" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InTable, token);
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/#clear-the-stack-back-to-a-table-row-context
    fn clear_the_stack_back_to_a_table_row_context(&mut self) {
        // SPEC: When the steps above require the UA to clear the stack back to a table row context,
        //       it means that the UA must, while the current node is not a tr, template, or html element,
        //       pop elements from the stack of open elements.
        while !self.current_node_is_one_of_elements_with_tag(&["tr", "template", "html"]) {
            self.stack_of_open_elements.pop_current_element();
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-afterbody
    fn handle_after_body(&mut self, token: &mut Token) {
        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                // SPEC: Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody, token);
            }
            Token::Comment { .. } => todo!(),
            Token::Doctype { .. } => {
                // SPEC: Parse error. Ignore the token.
                log_parser_error!();
            }
            Token::StartTag { name, .. } if name == "html" => {
                // SPEC: Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody, token);
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
                self.reprocess_token(token);
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#the-after-after-body-insertion-mode
    fn handle_after_after_body(&mut self, token: &mut Token) {
        macro_rules! process_token {
            () => {
                // SPEC: Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody, token);
            };
        }

        match token {
            Token::Comment { .. } => todo!(),
            Token::Doctype { .. } => {
                process_token!();
            }
            Token::Character { data } if is_parser_whitespace(*data) => {
                process_token!();
            }
            Token::StartTag { name, .. } if name == "html" => {
                process_token!();
            }
            Token::EndOfFile => {
                // SPEC: Stop parsing.
                self.stop_parsing()
            }
            _ => {
                // SPEC: Parse error.
                log_parser_error!();

                // SPEC: Switch the insertion mode to "in body" and reprocess the token.
                self.switch_insertion_mode_to(InsertionMode::InBody);
                self.reprocess_token(token);
            }
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inforeign
    fn process_token_using_the_rules_for_foreign_content(&mut self, token: &mut Token) {
        eprintln!(
            "\x1b[32m[Parser::InsertionMode::{:?}] {:?}\x1b[0m",
            self.insertion_mode, token
        );

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
                self.reprocess_token(token);
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
                    token.acknowledge_self_closing_flag_if_set();
                    self.stack_of_open_elements.pop_current_element();
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
        self.stack_of_open_elements.clear();
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
        while let Some(token) = self.tokenizer.next_token() {
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
            if self.stack_of_open_elements.is_empty()
                || match &self.adjusted_current_node().unwrap().data {
                    NodeData::Element { name, .. } => name.namespace == Some(Namespace::Html),
                    _ => false,
                }
                || matches!(token, Token::EndOfFile)
            {
                self.process_token_using_the_rules_for(self.insertion_mode, &mut token);
            } else {
                self.process_token_using_the_rules_for_foreign_content(&mut token)
            }
        }

        self.document
    }
}
