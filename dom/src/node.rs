use std::cell::{Cell, RefCell};

use crate::arena::{NodeLink, NodeRef};
use crate::dom_exception::DomException;
use crate::{Attribute, Namespace, QualifiedName};

/// A HTML Node.
#[derive(Eq, PartialOrd, Ord, Clone)]
pub struct Node<'a> {
    document: NodeLink<'a>,
    parent: NodeLink<'a>,
    children: RefCell<Vec<NodeRef<'a>>>,
    previous_sibling: NodeLink<'a>,
    next_sibling: NodeLink<'a>,
    first_child: NodeLink<'a>,
    last_child: NodeLink<'a>,
    pub data: NodeData,
}

impl PartialEq for Node<'_> {
    fn eq<'a>(&'a self, other: &'a Self) -> bool {
        let data_matches = match (&self.data, &other.data) {
            (
                NodeData::Doctype {
                    name: a_name,
                    public_id: a_public_id,
                    system_id: a_system_id,
                },
                NodeData::Doctype {
                    name: b_name,
                    public_id: b_public_id,
                    system_id: b_system_id,
                },
            ) => a_name == b_name && a_public_id == b_public_id && a_system_id == b_system_id,
            (
                NodeData::Element {
                    name: a_name,
                    namespace: a_namespace,
                    attributes: a_attributes,
                },
                NodeData::Element {
                    name: b_name,
                    namespace: b_namespace,
                    attributes: b_attributes,
                },
            ) => a_name == b_name && a_namespace == b_namespace && a_attributes == b_attributes,

            (
                NodeData::Attr {
                    namespace: a_namespace,
                    local_name: a_local_name,
                    value: a_value,
                    ..
                },
                NodeData::Attr {
                    namespace: b_namespace,
                    local_name: b_local_name,
                    value: b_value,
                    ..
                },
            ) => a_namespace == b_namespace && a_local_name == b_local_name && a_value == b_value,
            (
                NodeData::CharacterData {
                    data: a_data,
                    variant: a_variant,
                },
                NodeData::CharacterData {
                    data: b_data,
                    variant: b_variant,
                },
            ) => a_data == b_data && a_variant == b_variant,
            (a, b) => a == b,
        };

        data_matches
            && (self.children.clone().into_inner().len()
                == other.children.clone().into_inner().len())
            && self
                .children
                .clone()
                .into_inner()
                .iter()
                .zip(other.children.clone().into_inner().iter())
                .all(|(a, b)| Node::are_same(*a, *b))
    }
}

impl<'a> Node<'a> {
    pub fn new(document: Option<NodeRef<'a>>, data: NodeData) -> Node<'a> {
        Node {
            document: Cell::new(document),
            parent: Cell::new(None),
            children: RefCell::new(Vec::new()),
            previous_sibling: Cell::new(None),
            next_sibling: Cell::new(None),
            first_child: Cell::new(None),
            last_child: Cell::new(None),
            data,
        }
    }

    pub fn root(&'a self) -> NodeRef<'a> {
        // SPEC: The root of an object is itself, if its parent is null,
        //       or else it is the root of its parent.
        //       The root of a tree is any object participating in that tree whose parent is null.
        // FIXME: Is the root always the document?
        self.document()
    }

    pub fn are_same(a: NodeRef<'a>, b: NodeRef<'a>) -> bool {
        a == b
    }

    pub fn are_same_optional(a: Option<NodeRef<'a>>, b: Option<NodeRef<'a>>) -> bool {
        if let Some(a) = a {
            if let Some(b) = b {
                return Node::are_same(a, b);
            }
        }

        a.is_none() && b.is_none()
    }

    pub fn have_same_root(a: NodeRef<'a>, b: NodeRef<'a>) -> bool {
        Node::are_same(a.root(), b.root())
    }

    pub fn is_following(&'a self, _other: NodeRef<'a>) -> bool {
        // SPEC: An object A is following an object B if A and B are
        //       in the same tree and A comes after B in tree order.
        todo!()
    }

    pub fn is_preceding(&'a self, _other: NodeRef<'a>) -> bool {
        // SPEC: An object A is following an object B if A and B are
        //       in the same tree and A comes after B in tree order.
        todo!()
    }

    pub fn is_ancestor_of(&self, _other: NodeRef<'a>) -> bool {
        todo!()
    }

    pub fn is_child_of(&self, _other: NodeRef<'a>) -> bool {
        todo!()
    }

    pub fn document(&'a self) -> NodeRef<'a> {
        match self.document.get() {
            Some(document) => document,
            None => self,
        }
    }

    pub fn parent(&self) -> Option<NodeRef<'a>> {
        self.parent.get()
    }

    pub fn previous_sibling(&self) -> Option<NodeRef<'a>> {
        self.previous_sibling.get()
    }

    pub fn next_sibling(&self) -> Option<NodeRef<'a>> {
        self.next_sibling.get()
    }

    pub fn first_child(&self) -> Option<NodeRef<'a>> {
        self.first_child.get()
    }

    pub fn last_child(&self) -> Option<NodeRef<'a>> {
        self.last_child.get()
    }

    pub fn children(&'a self) -> std::cell::Ref<Vec<NodeRef<'a>>> {
        self.children.borrow()
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-tree-inclusive-ancestor
    pub fn inclusive_anscestors(&'a self) -> Vec<NodeRef<'a>> {
        let mut nodes = Vec::new();
        let mut current = self.parent();
        while let Some(inclusive_anscestor) = current {
            nodes.push(inclusive_anscestor);
            current = inclusive_anscestor.parent();
        }
        nodes
    }

    pub fn is_document_fragment(&self) -> bool {
        matches!(self.data, NodeData::DocumentFragment)
    }

    pub fn is_comment(&self) -> bool {
        matches!(
            self.data,
            NodeData::CharacterData {
                variant: CharacterDataVariant::Comment,
                ..
            }
        )
    }

    pub fn is_doctype(&self) -> bool {
        matches!(self.data, NodeData::Doctype { .. })
    }

    pub fn is_document(&self) -> bool {
        matches!(self.data, NodeData::Document)
    }

    pub fn is_element(&self) -> bool {
        matches!(self.data, NodeData::Element { .. })
    }

    pub fn is_character_data(&self) -> bool {
        matches!(self.data, NodeData::CharacterData { .. })
    }

    pub fn is_processing_instruction(&self) -> bool {
        matches!(
            self.data,
            NodeData::CharacterData {
                variant: CharacterDataVariant::ProcessingInstruction { .. },
                ..
            }
        )
    }

    pub fn is_text(&self) -> bool {
        matches!(
            self.data,
            NodeData::CharacterData {
                variant: CharacterDataVariant::Text { .. },
                ..
            }
        )
    }

    pub fn is_attr(&self) -> bool {
        matches!(self.data, NodeData::Attr { .. })
    }

    pub fn is_element_with_one_of_tags(&self, tags: &[&str]) -> bool {
        if let Some(name) = self.element_tag_name() {
            return tags.contains(&name);
        }
        false
    }

    pub fn is_element_with_tag(&self, tag: &str) -> bool {
        if let Some(name) = self.element_tag_name() {
            return name == tag;
        }
        false
    }

    pub fn is_marker_element(&self) -> bool {
        self.is_element_with_one_of_tags(&[
            "applet", "object", "marquee", "template", "td", "th", "caption",
        ])
    }

    pub fn is_special_tag(&self) -> bool {
        self.is_element_with_one_of_tags(&[
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
            "mi",
            "mo",
            "mn",
            "ms",
            "mtext",
            "annotation-xml",
            "foreignObject",
            "desc",
            "title",
        ])
    }

    pub fn length(&'a self) -> usize {
        if self.is_doctype() || self.is_attr() {
            0
        } else if let NodeData::CharacterData { data, .. } = self.data.clone() {
            return data.borrow().len();
        } else {
            self.children().len()
        }
    }

    pub fn element_tag_name(&self) -> Option<&str> {
        if let NodeData::Element { name, .. } = &self.data {
            return Some(&name.local);
        }
        None
    }

    pub fn dump(&'a self) {
        self.internal_dump("");
    }

    fn internal_dump(&'a self, indentation: &str) {
        let indent = "  ";

        let (opening_tag, closing_tag) = self.data.tags();
        if let Some(opening_tag) = opening_tag {
            println!("{indentation}{opening_tag}");
        }

        for child in self.children().iter() {
            let mut indentation = indentation.to_string();
            indentation.push_str(indent);
            child.internal_dump(&indentation);
        }
        if let Some(closing_tag) = closing_tag {
            println!("{indentation}{closing_tag}");
        }
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-tree-index
    pub fn index(&'a self) -> usize {
        let mut index = 0;
        let mut current = self.previous_sibling();
        while let Some(node) = current {
            index += 1;
            current = node.next_sibling()
        }
        index
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-descendant
    fn shadow_including_inclusive_descendants(&'a self) -> std::cell::Ref<Vec<NodeRef<'a>>> {
        // FIXME: Currently we assume every node is an inclusive descendant of the shadow root
        self.children()
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-remove
    pub fn remove(&'a self, _node: &'a Self) {
        todo!()
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-adopt
    pub fn adopt(&'a self, node: &'a Self) {
        if !self.is_document() {
            panic!("only the Document Node should adopt nodes");
        }

        // SPEC: 1. Let oldDocument be node’s node document.
        let old_document = node.document();

        // SPEC: 2. If node’s parent is non-null, then remove node.
        if node.parent().is_some() {
            self.document().remove(node);
        }

        // SPEC: 3. If document is not oldDocument, then:
        if std::ptr::eq::<Node>(self.document(), old_document) {
            // SPEC: 3.1. For each inclusiveDescendant in node’s shadow-including inclusive descendants:
            for inclusive_descendant in node.shadow_including_inclusive_descendants().iter() {
                // SPEC: 3.1.1. Set inclusiveDescendant’s node document to document.
                inclusive_descendant.document.set(Some(self.document()));

                // SPEC: 3.1.2. If inclusiveDescendant is an element,
                //              then set the node document of each attribute in inclusiveDescendant’s attribute list to document.
                if inclusive_descendant.is_element() {
                    todo!()
                }
            }

            // SPEC: 3.2. For each inclusiveDescendant in node’s shadow-including inclusive descendants that is custom,
            //            enqueue a custom element callback reaction with inclusiveDescendant, callback name "adoptedCallback",
            //            and an argument list containing oldDocument and document.
            // FIXME: Implement

            // SPEC: 3.3. For each inclusiveDescendant in node’s shadow-including inclusive descendants,
            //            in shadow-including tree order, run the adopting steps with inclusiveDescendant and oldDocument.
            // FIXME: Implement
        }
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-append
    pub fn append(&'a self, child: &'a Self) {
        self.pre_insert(child, None);
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity
    pub fn ensure_pre_insertion_validity(
        &'a self,
        node: NodeRef<'a>,
        child: Option<NodeRef<'a>>,
    ) -> Result<(), DomException> {
        // SPEC: 1. If parent is not a Document, DocumentFragment, or Element node, then throw a "HierarchyRequestError" DOMException.
        if !self.is_document() && !self.is_document_fragment() && !self.is_element() {
            return Err(DomException::HierarchyRequestError);
        }

        // SPEC: 2. If node is a host-including inclusive ancestor of parent, then throw a "HierarchyRequestError" DOMException.
        // FIXME: Implement

        // SPEC: 3. If child is non-null and its parent is not parent, then throw a "NotFoundError" DOMException.
        if let Some(Some(child_parent)) = child.map(|c| c.parent()) {
            if !Node::are_same(child_parent, self) {
                return Err(DomException::NotFoundError);
            }
        }

        // SPEC: 4. If node is not a DocumentFragment, DocumentType, Element, or CharacterData node, then throw a "HierarchyRequestError" DOMException.
        if !node.is_document_fragment()
            && !node.is_doctype()
            && !node.is_element()
            && !node.is_character_data()
        {
            return Err(DomException::HierarchyRequestError);
        }
        // SPEC: 5. If either node is a Text node and parent is a document,
        //          or node is a doctype and parent is not a document,
        //          then throw a "HierarchyRequestError" DOMException.
        if (node.is_text() && self.is_document()) || (node.is_doctype() && !self.is_document()) {
            return Err(DomException::HierarchyRequestError);
        }

        // SPEC: 6. If parent is a document, and any of the statements below,
        //          switched on the interface node implements, are true, then throw a "HierarchyRequestError" DOMException.
        if self.is_document()
            && match node.data {
                // FIXME: Implement DocumentFragment
                NodeData::Element { .. } => {
                    // SPEC: parent has an element child,
                    self.children().iter().any(|child| child.is_element()) ||
                        // SPEC: child is a doctype,
                        child.is_some_and(|c|c.is_doctype()) ||
                        // SPEC: or child is non-null and a doctype is following child.
                        child.is_some_and(|c|c.next_sibling().is_some_and(|c|c.is_doctype()))
                }
                NodeData::Doctype { .. } => {
                    // SPEC: parent has a doctype child,
                    self.children().iter().any(|child| child.is_doctype()) ||
                        // SPEC: child is non-null and an element is preceding child,
                        child.is_some_and(|c|c.previous_sibling().is_some_and(|c|c.is_element())) ||
                        // SPEC: or child is null and parent has an element child.
                        child.is_none() && self.children().iter().any(|child| child.is_element())
                }
                _ => false,
            }
        {
            return Err(DomException::HierarchyRequestError);
        };

        Ok(())
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-insert
    pub fn insert_before(&'a self, node: &'a Self, child: Option<&'a Self>) {
        // SPEC: 1. Let nodes be node’s children, if node is a DocumentFragment node; otherwise « node ».
        let mut nodes = Vec::new();
        match node.is_document_fragment() {
            true => {
                for child in node.children().iter() {
                    nodes.push(*child);
                }
            }
            false => nodes.push(node),
        }

        // SPEC: 2. Let count be nodes’s size.
        let count = nodes.len();

        // SPEC: 3. If count is 0, then return.
        if count == 0 {
            return;
        }

        // SPEC: 4. If node is a DocumentFragment node, then:
        // FIXME: Implement

        // SPEC: 5. If child is non-null, then:
        if let Some(_child) = child {
            // SPEC: 5.1. For each live range whose start node is parent and start offset is greater than child’s index, increase its start offset by count.
            // SPEC: 5.2. For each live range whose end node is parent and end offset is greater than child’s index, increase its end offset by count.
            // FIXME: Implement
        }

        // SPEC: 6. Let previousSibling be child’s previous sibling or parent’s last child if child is null.
        let _previous_sibling = match child {
            Some(child) => child.previous_sibling(),
            None => self.last_child(),
        };

        // SPEC: 7. For each node in nodes, in tree order:
        for node in nodes.iter() {
            // SPEC: 7.1. Adopt node into parent’s node document.
            self.document().adopt(node);

            if let Some(child) = child {
                // SPEC: 7.3. Otherwise, insert node into parent’s children before child’s index.
                self.children.borrow_mut().insert(child.index(), node);

                node.previous_sibling.set(child.previous_sibling());
                node.next_sibling.set(Some(child));
                if let Some(child_previous_sibling) = child.previous_sibling.get() {
                    child_previous_sibling.next_sibling.set(Some(node));
                }
                if Node::are_same_optional(self.first_child.get(), Some(child)) {
                    self.first_child.set(Some(node));
                }

                child.previous_sibling.set(Some(node));

                node.parent.set(Some(self));
            } else {
                // SPEC: 7.2. If child is null, then append node to parent’s children.
                self.children.borrow_mut().push(node);

                if self.last_child().is_some() {
                    self.last_child.set(Some(node));
                }
                node.previous_sibling.set(self.last_child());
                node.parent.set(Some(self));
                self.last_child.set(Some(node));
                if self.first_child().is_none() {
                    self.first_child.set(self.last_child());
                }
            }

            // SPEC: 7.4. If parent is a shadow host whose shadow root’s slot assignment is "named" and node is a slottable, then assign a slot for node.
            // FIXME: Implement

            // SPEC: 7.5. If parent’s root is a shadow root, and parent is a slot whose assigned nodes is the empty list, then run signal a slot change for parent.
            // FIXME: Implement

            // SPEC: 7.6. Run assign slottables for a tree with node’s root.
            // FIXME: Implement

            // SPEC: 7.7. For each shadow-including inclusive descendant inclusiveDescendant of node, in shadow-including tree order:
            // FIXME: Implement
        }

        // SPEC: 8. If suppress observers flag is unset, then queue a tree mutation record for parent with nodes, « », previousSibling, and child.
        // FIXME: Implement

        // SPEC: 9. Run the children changed steps for parent.
        // FIXME: Implement
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-pre-insert
    fn pre_insert(&'a self, node: &'a Self, child: Option<&'a Self>) -> &'a Self {
        // SPEC: 1. Ensure pre-insertion validity of node into parent before child.
        // FIXME: Implement

        // SPEC: 2. Let referenceChild be child.
        let mut reference_child = child;

        // SPEC: 3. If referenceChild is node, then set referenceChild to node’s next sibling.
        if reference_child == Some(node) {
            reference_child = node.next_sibling()
        }

        // SPEC: 4. Insert node into parent before referenceChild.
        self.insert_before(node, reference_child);

        // SPEC: 5. Return node.
        node
    }
}

impl<'a> std::fmt::Debug for Node<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // destructuring will make it fail to compile
        // if you later add a field and forget to update here
        let Node {
            document,
            parent,
            children,
            previous_sibling,
            next_sibling,
            first_child,
            last_child,
            data,
        } = self;

        if self.is_document() {
            return write!(f, "Document {{ ... }}");
        }

        // impl Debug for HexArray here
        f.debug_struct("Node")
            .field("data", &data)
            .field("children", &children.borrow())
            .field(
                "previous_sibling",
                &previous_sibling.get().map(|v| v.data.to_string()),
            )
            .field(
                "next_sibling",
                &next_sibling.get().map(|v| v.data.to_string()),
            )
            .field(
                "first_child",
                &first_child.get().map(|v| v.data.to_string()),
            )
            .field("last_child", &last_child.get().map(|v| v.data.to_string()))
            .field("parent", &parent.get().map(|v| v.data.to_string()))
            .field("document", &document.get().map(|v| v.data.to_string()))
            .finish()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum NodeData {
    Document,
    DocumentFragment,
    Element {
        name: QualifiedName,
        namespace: Option<Namespace>,
        attributes: RefCell<Vec<Attribute>>,
    },
    Doctype {
        name: String,
        public_id: String,
        system_id: String,
    },
    CharacterData {
        data: RefCell<String>,
        variant: CharacterDataVariant,
    },
    Attr {
        namespace: Option<Namespace>,
        prefix: Option<String>,
        local_name: String,
        name: String,
        value: String,
        owner_element: Option<()>,
    },
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum CharacterDataVariant {
    Text,
    ProcessingInstruction { target: String },
    Comment,
}

impl NodeData {
    pub fn tags(&self) -> (Option<String>, Option<String>) {
        let std_close = Some("\"".to_string());

        match self {
            NodeData::Document => (Some("Document".to_string()), std_close),
            NodeData::Doctype { name, .. } => (Some(format!("<!DOCTYPE {name}>")), None),
            NodeData::CharacterData {
                variant: CharacterDataVariant::Text { .. },
                data,
            } => (Some(format!("#text \"{}\"", data.borrow().trim())), None),
            NodeData::Element { name, .. } => (Some(format!("<{}>", name.local)), std_close),
            _ => (None, None),
        }
    }
}

impl ToString for NodeData {
    fn to_string(&self) -> String {
        match self {
            NodeData::Document => "Document".to_string(),
            NodeData::Doctype { name, .. } => format!("DOCTYPE {name}"),
            NodeData::Element { name, .. } => {
                format!("Element({})", name.local)
            }
            NodeData::CharacterData { variant, .. } => match variant {
                CharacterDataVariant::Text { .. } => "Text".to_string(),
                CharacterDataVariant::ProcessingInstruction { .. } => {
                    "ProcessingInstruction".to_string()
                }
                CharacterDataVariant::Comment { .. } => "Comment".to_string(),
            },
            NodeData::DocumentFragment => "DocumentFragment".to_string(),
            NodeData::Attr { .. } => "Attr".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::node::{Node, NodeData};

    #[test]
    fn are_same_optional() {
        let a = &Node::new(None, NodeData::Document);
        let b = &Node::new(None, NodeData::Document);

        assert_eq!(Node::are_same_optional(Some(a), Some(b)), false);
        assert_eq!(Node::are_same_optional(Some(a), None), false);
        assert_eq!(Node::are_same_optional(None, Some(b)), false);
        assert_eq!(Node::are_same_optional(Some(a), Some(a)), true);
        assert_eq!(Node::are_same_optional(None, None), true);
    }

    #[test]
    fn are_same() {
        let a = &Node::new(None, NodeData::Document);
        let b = &Node::new(None, NodeData::Document);

        assert_eq!(Node::are_same(a, b), false);
        assert_eq!(Node::are_same(b, a), false);
        assert_eq!(Node::are_same(a, a), true);
    }
}
