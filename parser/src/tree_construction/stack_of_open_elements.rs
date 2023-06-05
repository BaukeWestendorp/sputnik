use std::cell::RefCell;

use crate::types::NodeRef;

pub struct StackOfOpenElements<'a> {
    pub(crate) elements: RefCell<Vec<NodeRef<'a>>>,
}

pub(super) static SPECIAL_TAGS: &[&str] = &[
    "address",
    "applet",
    "area",
    "article",
    "aside",
    "base",
    "basefont",
    "bgsound",
    "blockquote",
    "body",
    "br",
    "button",
    "caption",
    "center",
    "col",
    "colgroup",
    "dd",
    "details",
    "dir",
    "div",
    "dl",
    "dt",
    "embed",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "frame",
    "frameset",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hgroup",
    "hr",
    "html",
    "iframe",
    "img",
    "input",
    "keygen",
    "li",
    "link",
    "listing",
    "main",
    "marquee",
    "menu",
    "meta",
    "nav",
    "noembed",
    "noframes",
    "noscript",
    "object",
    "ol",
    "p",
    "param",
    "plaintext",
    "pre",
    "script",
    "search",
    "section",
    "select",
    "source",
    "style",
    "summary",
    "table",
    "tbody",
    "td",
    "template",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "title",
    "tr",
    "track",
    "ul",
    "wbr",
    "xmp",
    // FIXME: Implement MathML mi
    // FIXME: Implement MathML mo
    // FIXME: Implement MathML mn
    // FIXME: Implement MathML ms
    // FIXME: Implement MathML mtext
    // FIXME: Implement MathML annotation-xml
    // FIXME: Implement SVG foreignObject
    // FIXME: Implement SVG desc
    // FIXME: Implement SVG title
];

pub(super) static BASE_SCOPE_TAGS: &[&str] = &[
    "applet",
    "caption",
    "html",
    "table",
    "td",
    "th",
    "marquee",
    "object",
    "template",
    "mi",
    "mo",
    "mn",
    "ms",
    "mtext",
    "annotation-xml",
    "foreignObject",
    "desc",
    "title",
];

impl<'a> StackOfOpenElements<'a> {
    pub fn new() -> Self {
        Self {
            elements: RefCell::new(vec![]),
        }
    }

    // https://html.spec.whatwg.org/#current-node
    pub fn current_node(&self) -> Option<NodeRef<'a>> {
        self.elements.borrow().last().copied()
    }

    // https://html.spec.whatwg.org/#adjusted-current-node
    pub fn adjusted_current_node(&self) -> Option<NodeRef<'a>> {
        // FIXME: Implement
        self.current_node()
    }

    pub fn first(&self) -> Option<NodeRef<'a>> {
        self.elements.borrow().first().copied()
    }

    pub fn push(&self, element: NodeRef<'a>) {
        self.elements.borrow_mut().push(element);
    }

    pub fn pop(&self) {
        self.elements.borrow_mut().pop();
    }

    pub fn pop_elements_until_element_with_tag_name_has_been_popped(&self, tag_name: &str) {
        while !self.current_node().unwrap().is_element_with_tag(tag_name) {
            self.pop();
        }
        self.pop()
    }

    pub fn pop_elements_until_element_has_been_popped(&self, node: NodeRef<'a>) {
        while self.current_node() != Some(node) {
            self.pop();
        }
        self.pop()
    }

    pub fn insert_immediately_below(&self, element: NodeRef<'a>, target: NodeRef<'a>) {
        if let Some(index) = self.elements.borrow().iter().position(|e| e == &target) {
            self.elements.borrow_mut().insert(index + 1, element);
        }
    }

    pub fn replace(&self, target: NodeRef<'a>, replacement: NodeRef<'a>) {
        if let Some(index) = self.elements.borrow().iter().position(|e| e == &target) {
            self.elements.borrow_mut()[index] = replacement;
        }
    }

    pub fn remove_element(&self, element: NodeRef<'a>) {
        if let Some(index) = self.elements.borrow().iter().position(|e| e == &element) {
            self.elements.borrow_mut().remove(index);
        }
    }

    pub fn element_immediately_above(&self, target: NodeRef<'a>) -> Option<NodeRef<'a>> {
        let mut found = false;
        for element in self.elements.borrow().iter().rev() {
            if *element == target {
                found = true;
            } else if found {
                return Some(element);
            }
        }
        None
    }

    pub fn topmost_special_node_below(&self, target: NodeRef<'a>) -> Option<NodeRef<'a>> {
        let mut best = None;
        for element in self.elements.borrow().iter().rev() {
            if *element == target {
                break;
            }
            if element.is_element_with_one_of_tags(SPECIAL_TAGS) {
                best = Some(*element);
            }
        }
        best
    }

    pub fn contains(&self, target: NodeRef<'a>) -> bool {
        self.elements
            .borrow()
            .iter()
            .any(|element| *element == target)
    }

    pub fn contains_one_of_tags(&self, tags: &[&str]) -> bool {
        self.elements
            .borrow()
            .iter()
            .any(|node| node.is_element_with_one_of_tags(tags))
    }

    pub fn is_empty(&self) -> bool {
        self.elements.borrow().is_empty()
    }

    pub fn clear(&self) {
        self.elements.borrow_mut().clear()
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-the-specific-scope
    fn has_tag_name_in_scope(&self, target: &str, list: &[&str]) -> bool {
        // 1. Initialize node to be the current node (the bottommost node of the stack).
        for node in self.elements.borrow().iter().rev() {
            // 2. If node is the target node, terminate in a match state.
            if node.is_element_with_tag(target) {
                return true;
            }
            // 3. Otherwise, if node is one of the element types in list, terminate in a failure state.
            if node.is_element_with_one_of_tags(list) {
                return false;
            }
            // 4. Otherwise, set node to the previous entry in the stack of open elements and return to step 2. (This will never fail, since the loop will always terminate in the previous step if the top of the stack — an html element — is reached.)
        }

        unreachable!();
    }

    pub fn has_element_in_scope(&self, target_node: NodeRef<'a>) -> bool {
        // 1. Initialize node to be the current node (the bottommost node of the stack).
        for node in self.elements.borrow().iter().rev() {
            // 2. If node is the target node, terminate in a match state.
            if *node == target_node {
                return true;
            }
            // 3. Otherwise, if node is one of the element types in list, terminate in a failure state.
            if node.is_element_with_one_of_tags(BASE_SCOPE_TAGS) {
                return false;
            }

            // 4. Otherwise, set node to the previous entry in the stack of open elements and return to step 2. (This will never fail, since the loop will always terminate in the previous step if the top of the stack — an html element — is reached.)
        }

        unreachable!();
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-scope
    pub fn has_element_with_tag_name_in_scope(&self, tag_name: &str) -> bool {
        self.has_tag_name_in_scope(tag_name, BASE_SCOPE_TAGS)
    }

    // https://html.spec.whatwg.org/#has-an-element-in-list-item-scope
    pub fn has_element_with_tag_name_in_list_item_scope(&self, tag_name: &str) -> bool {
        self.has_tag_name_in_scope(tag_name, &[BASE_SCOPE_TAGS, &["ol", "ul"]].concat())
    }

    // https://html.spec.whatwg.org/#has-an-element-in-button-scope
    pub fn has_element_with_tag_name_in_button_scope(&self, tag_name: &str) -> bool {
        self.has_tag_name_in_scope(tag_name, &[BASE_SCOPE_TAGS, &["button"]].concat())
    }
}
