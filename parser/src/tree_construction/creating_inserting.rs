use std::cell::Cell;

use tokenizer::Token;

use crate::dom::Node;
use crate::namespace::Namespace;
use crate::types::NodeRef;
use crate::Parser;

enum InsertionLocation {
    AfterLastChildIfAny,
}

pub(crate) struct AdjustedInsertionLocation<'a> {
    parent: NodeRef<'a>,
    child: InsertionLocation,
}

impl<'a> AdjustedInsertionLocation<'a> {
    pub(crate) fn child_node(&self) -> Option<NodeRef<'a>> {
        match self.child {
            InsertionLocation::AfterLastChildIfAny => None,
        }
    }

    pub(crate) fn insert(&self, element: NodeRef<'a>) {
        Node::insert(element, self.parent, self.child_node(), false);
    }
}

impl<'a> Parser<'a> {
    // https://html.spec.whatwg.org/multipage/parsing.html#appropriate-place-for-inserting-a-node
    pub(crate) fn appropriate_place_for_inserting_node(
        &'a self,
        override_target: Option<NodeRef<'a>>,
    ) -> AdjustedInsertionLocation<'a> {
        let target = match override_target {
            // 1. If there was an override target specified, then let target be the override target.
            Some(override_target) => override_target,
            // Otherwise, let target be the current node.
            None => self
                .open_elements
                .current_node()
                .expect("There will always be an html element on the stack"),
        };

        // 2. Determine the adjusted insertion location using the first matching steps from the following list:
        // FIXME: If foster parenting is enabled and target is a table, tbody, tfoot, thead, or tr element
        // Otherwise, let adjusted insertion location be inside target, after its last child (if any).
        let adjusted_insertion_location = AdjustedInsertionLocation {
            parent: target,
            child: InsertionLocation::AfterLastChildIfAny,
        };

        // FIXME: 3. If the adjusted insertion location is inside a template element, let it instead be inside the template element's template contents, after its last child (if any).
        // 4. Return the adjusted insertion location.
        adjusted_insertion_location
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#create-an-element-for-the-token
    pub(super) fn create_element_for_token(
        &'a self,
        token: &Token,
        namespace: Namespace,
        intended_parent: NodeRef<'a>,
    ) -> NodeRef<'a> {
        // FIXME: 1. If the active speculative HTML parser is not null, then return the result of creating a speculative mock element given given namespace, the tag name of the given token, and the attributes of the given token.
        // FIXME: 2. Otherwise, optionally create a speculative mock element given given namespace, the tag name of the given token, and the attributes of the given token.

        // 3. Let document be intended parent's node document.
        let document = intended_parent.node_document();

        // 4. Let local name be the tag name of the token.
        let local_name = match token {
            Token::StartTag {
                name: local_name, ..
            } => {
                // FIXME: 5. Let is be the value of the "is" attribute in the given token, if such an attribute exists, or null otherwise.
                // FIXME: 6. Let definition be the result of looking up a custom element definition given document, given namespace, local name, and is.
                // FIXME: 7. If definition is non-null and the parser was not created as part of the HTML fragment parsing algorithm, then let will execute script be true. Otherwise, let it be false.
                // FIXME: 8. If will execute script is true, then:
                local_name
            }
            _ => panic!("cannot create element from non-StartTag token"),
        };

        // 9. Let element be the result of creating an element given document, localName, given namespace, null, and is. If will execute script is true, set the synchronous custom elements flag; otherwise, leave it unset.
        let element = self.create_element(document, local_name, namespace, None, None, false);

        // FIXME: 10. Append each attribute in the given token to element.
        // FIXME: 11. If will execute script is true, then:
        // FIXME: 12. If element has an xmlns attribute in the XMLNS namespace whose value is not exactly the same as the element's namespace, that is a parse error. Similarly, if element has an xmlns:xlink attribute in the XMLNS namespace whose value is not the XLink Namespace, that is a parse error.
        // FIXME: 13. If element is a resettable element, invoke its reset algorithm. (This initializes the element's value and checkedness based on the element's attributes.)
        // FIXME: 14. If element is a form-associated element and not a form-associated custom element, the form element pointer is not null, there is no template element on the stack of open elements, element is either not listed or doesn't have a form attribute, and the intended parent is in the same tree as the element pointed to by the form element pointer, then associate element with the form element pointed to by the form element pointer and set element's parser inserted flag.

        element
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#insert-a-foreign-element
    pub(crate) fn insert_foreign_element_for_token(
        &'a self,
        token: &Token,
        namespace: Namespace,
    ) -> NodeRef<'a> {
        // 1. Let the adjusted insertion location be the appropriate place for inserting a node.
        let adjusted_insertion_location = self.appropriate_place_for_inserting_node(None);

        // 2. Let element be the result of creating an element for the token in the given namespace, with the intended parent being the element in which the adjusted insertion location finds itself.
        let element =
            self.create_element_for_token(token, namespace, adjusted_insertion_location.parent);

        // 3. If it is possible to insert element at the adjusted insertion location, then:

        match Node::ensure_pre_insertion_validity(
            element,
            adjusted_insertion_location.parent,
            adjusted_insertion_location.child_node(),
        )
        .is_ok()
        {
            true => {
                // 3.1. FIXME: If the parser was not created as part of the HTML fragment parsing algorithm, then push a new element queue onto element's relevant agent's custom element reactions stack.

                // 3.2. Insert element at the adjusted insertion location.
                adjusted_insertion_location.insert(element);

                // 3.3. FIXME: If the parser was not created as part of the HTML fragment parsing algorithm, then pop the element queue from element's relevant agent's custom element reactions stack, and invoke custom element reactions in that queue.
            }
            false => (),
        }

        // 4. Push element onto the stack of open elements so that it is the new current node.
        self.open_elements.push(element);

        // 5. Return element.
        element
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#insert-an-html-element
    pub(crate) fn insert_html_element_for_token(&'a self, token: &Token) -> NodeRef<'a> {
        self.insert_foreign_element_for_token(token, Namespace::Html)
    }

    pub(crate) fn insert_html_element_for_start_tag(&'a self, tag: &str) -> NodeRef<'a> {
        self.insert_html_element_for_token(&Token::StartTag {
            name: tag.to_string(),
            self_closing: false,
            self_closing_acknowledged: Cell::new(false),
            attributes: vec![],
        })
    }

    // https://html.spec.whatwg.org/#insert-a-character
    pub(crate) fn insert_character(&'a self, _character: char) {
        todo!()
    }

    // https://html.spec.whatwg.org/#insert-a-comment
    pub(crate) fn insert_comment(&'a self, _data: &str) {
        todo!()
    }
}
