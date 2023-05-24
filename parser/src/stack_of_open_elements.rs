use dom::arena::NodeRef;
use dom::node::{Node, NodeData};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct StackOfOpenElements<'a> {
    pub elements: Vec<NodeRef<'a>>,
}

static BASE_SCOPE_ELEMENTS: &[&str] = &[
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
            elements: Vec::new(),
        }
    }

    pub fn current_node(&self) -> Option<NodeRef<'a>> {
        // SPEC: The current node is the bottommost node in this stack of open elements.
        self.elements.last().copied()
    }

    pub fn first(&self) -> Option<NodeRef<'a>> {
        self.elements.first().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn clear(&mut self) {
        self.elements.clear()
    }

    pub fn push(&mut self, element: NodeRef<'a>) {
        self.elements.push(element);
    }

    pub fn pop_current_element(&mut self) {
        self.elements.pop();
    }

    pub fn pop_elements_until_element_has_been_popped(&mut self, tag_name: &str) {
        let mut current = self.current_node();
        while let Some(NodeData::Element { name, .. }) = current.map(|c| &c.data) {
            self.pop_current_element();
            if name.local == tag_name {
                return;
            }
            current = self.current_node();
        }
    }

    pub fn remove_element(&mut self, element: NodeRef<'a>) {
        if let Some(index) = self.elements.iter().position(|e| e == &element) {
            self.elements.remove(index);
        }
    }

    pub fn element_immediately_above(&self, target: NodeRef<'a>) -> Option<NodeRef<'a>> {
        let mut found = false;
        for element in self.elements.iter().rev() {
            if Node::are_same(element, target) {
                found = true;
            } else if found {
                return Some(element);
            }
        }
        None
    }

    pub fn contains(&self, element: NodeRef<'a>) -> bool {
        self.elements.contains(&element)
    }

    pub fn contains_one_of_tags(&self, tags: &[&str]) -> bool {
        self.elements.iter().any(|element| {
            if let NodeData::Element { name, .. } = &element.data {
                return tags.contains(&name.local.as_str());
            }
            false
        })
    }

    pub fn last_with_tag(&self, tag: &str) -> Option<(usize, NodeRef<'a>)> {
        self.elements
            .iter()
            .rev()
            .enumerate()
            .find(|(_, element)| element.element_tag_name() == Some(tag))
            .map(|e| (e.0, *e.1))
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-the-specific-scope
    fn has_tag_name_in_scope(&self, target: &str, list: &[&str]) -> bool {
        // SPEC: 1. Initialize node to be the current node (the bottommost node of the stack).
        for node in self.elements.iter().rev() {
            // SPEC: 2. If node is the target node, terminate in a match state.
            if node.element_tag_name() == Some(target) {
                return true;
            }
            // SPEC: 3. Otherwise, if node is one of the element types in list, terminate in a failure state.
            if node.is_element_with_one_of_tags(list) {
                return false;
            }
            // SPEC: 4. Otherwise, set node to the previous entry in the stack of open elements and return to step 2. (This will never fail, since the loop will always terminate in the previous step if the top of the stack — an html element — is reached.)
        }
        unreachable!();
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-scope
    pub fn has_element_with_tag_name_in_scope(&self, tag_name: &str) -> bool {
        self.has_tag_name_in_scope(tag_name, BASE_SCOPE_ELEMENTS)
    }

    // SPECLINK: https://html.spec.whatwg.org/#has-an-element-in-button-scope
    pub fn has_element_with_tag_name_in_button_scope(&self, tag_name: &str) -> bool {
        self.has_tag_name_in_scope(tag_name, &[BASE_SCOPE_ELEMENTS, &["button"]].concat())
    }

    // SPECLINK: https://html.spec.whatwg.org/#has-an-element-in-table-scope
    pub fn has_element_with_tag_name_in_table_scope(&self, tag_name: &str) -> bool {
        self.has_tag_name_in_scope(tag_name, &["html", "table", "template"])
    }
}
