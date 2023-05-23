use std::cell::{Cell, RefCell};

use crate::arena::{Link, Ref};
use crate::dom_exception::DomException;
use crate::{Attribute, QualifiedName};

/// A HTML Node.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Node<'arena> {
    document: Link<'arena>,
    parent: Link<'arena>,
    children: RefCell<Vec<Ref<'arena>>>,
    previous_sibling: Link<'arena>,
    next_sibling: Link<'arena>,
    first_child: Link<'arena>,
    last_child: Link<'arena>,
    pub data: NodeData,
}

impl<'arena> Node<'arena> {
    pub fn new(document: Option<Ref<'arena>>, data: NodeData) -> Node<'arena> {
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

    pub fn document(&'arena self) -> Ref<'arena> {
        match self.document.get() {
            Some(document) => document,
            None => self,
        }
    }

    pub fn parent(&self) -> Option<Ref<'arena>> {
        self.parent.get()
    }

    pub fn previous_sibling(&self) -> Option<Ref<'arena>> {
        self.previous_sibling.get()
    }

    pub fn next_sibling(&self) -> Option<Ref<'arena>> {
        self.next_sibling.get()
    }

    pub fn first_child(&self) -> Option<Ref<'arena>> {
        self.first_child.get()
    }

    pub fn last_child(&self) -> Option<Ref<'arena>> {
        self.last_child.get()
    }

    pub fn children(&'arena self) -> std::cell::Ref<Vec<Ref<'arena>>> {
        self.children.borrow()
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-tree-inclusive-ancestor
    pub fn inclusive_anscestors(&'arena self) -> Vec<Ref<'arena>> {
        let mut nodes = Vec::new();
        let mut current = self.parent();
        while let Some(inclusive_anscestor) = current {
            nodes.push(inclusive_anscestor);
            current = inclusive_anscestor.parent();
        }
        nodes
    }

    pub fn is_document_fragment(&self) -> bool {
        // FIXME: Implement
        false
    }

    pub fn is_comment(&self) -> bool {
        // FIXME: This could be more efficient
        if let NodeData::Comment { .. } = self.data {
            return true;
        };
        false
    }

    pub fn is_doctype(&self) -> bool {
        // FIXME: This could be more efficient
        if let NodeData::Doctype { .. } = self.data {
            return true;
        };
        false
    }

    pub fn is_document(&self) -> bool {
        // FIXME: This could be more efficient
        if let NodeData::Document = self.data {
            return true;
        };

        false
    }

    pub fn is_element(&self) -> bool {
        // FIXME: This could be more efficient
        if let NodeData::Element { .. } = self.data {
            return true;
        };
        false
    }

    pub fn is_character_data(&self) -> bool {
        // FIXME: This could be more efficient
        // FIXME: CharacterData should be implemented
        self.is_text() || self.is_processing_instruction() || self.is_comment()
    }

    pub fn is_processing_instruction(&self) -> bool {
        // FIXME: This could be more efficient
        if let NodeData::ProcessingInstruction { .. } = self.data {
            return true;
        };
        false
    }

    pub fn is_text(&self) -> bool {
        // FIXME: This could be more efficient
        if let NodeData::Text { .. } = self.data {
            return true;
        };
        false
    }

    pub fn dump(&'arena self) {
        self.internal_dump("");
    }

    fn internal_dump(&'arena self, indentation: &str) {
        let (opening_tag, closing_tag) = self.data.tags();
        if let Some(opening_tag) = opening_tag {
            println!("{indentation}{opening_tag}");
        }
        for child in self.children().iter() {
            let mut indentation = indentation.to_string();
            indentation.push_str("    ");
            child.internal_dump(&indentation);
        }
        if let Some(closing_tag) = closing_tag {
            println!("{indentation}{closing_tag}");
        }
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-tree-index
    pub fn index(&'arena self) -> usize {
        let mut index = 0;
        let mut current = self.previous_sibling();
        while let Some(node) = current {
            index += 1;
            current = node.next_sibling()
        }
        index
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-descendant
    fn shadow_including_inclusive_descendants(&'arena self) -> std::cell::Ref<Vec<Ref<'arena>>> {
        // FIXME: Currently we assume every node is an inclusive descendant of the shadow root
        self.children()
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-remove
    pub fn remove(&'arena self, _node: &'arena Self) {
        todo!()
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-adopt
    pub fn adopt(&'arena self, node: &'arena Self) {
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
    pub fn append(&'arena self, child: &'arena Self) {
        self.pre_insert(child, None);
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity
    pub fn ensure_pre_insertion_validity(
        &'arena self,
        node: Ref<'arena>,
        child: Option<Ref<'arena>>,
    ) -> Result<(), DomException> {
        // SPEC: 1. If parent is not a Document, DocumentFragment, or Element node, then throw a "HierarchyRequestError" DOMException.
        if !self.is_document() && !self.is_document_fragment() && !self.is_element() {
            return Err(DomException::HierarchyRequestError);
        }

        // SPEC: 2. If node is a host-including inclusive ancestor of parent, then throw a "HierarchyRequestError" DOMException.
        // FIXME: Implement

        // SPEC: 3. If child is non-null and its parent is not parent, then throw a "NotFoundError" DOMException.
        if child.is_some_and(|c| c.parent() != Some(self)) {
            return Err(DomException::NotFoundError);
        }

        // SPEC: 4. If node is not a DocumentFragment, DocumentType, Element, or CharacterData node, then throw a "HierarchyRequestError" DOMException.
        if !node.is_document_fragment()
            && !node.is_doctype()
            && !node.is_element()
            && !node.is_character_data()
        {
            return Err(DomException::HierarchyRequestError);
        }
        // SPEC: 5. If either node is a Text node and parent is a document, or node is a doctype and parent is not a document, then throw a "HierarchyRequestError" DOMException.
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
    pub fn insert(&'arena self, node: &'arena Self, child: Option<&'arena Self>) {
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
            todo!()
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

            if let Some(_child) = child {
                // SPEC: 7.3. Otherwise, insert node into parent’s children before child’s index.
                todo!("Implement inserting child");
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
    fn pre_insert(&'arena self, node: &'arena Self, child: Option<&'arena Self>) -> &'arena Self {
        // SPEC: 1. Ensure pre-insertion validity of node into parent before child.
        // FIXME: Implement

        // SPEC: 2. Let referenceChild be child.
        let mut reference_child = child;

        // SPEC: 3. If referenceChild is node, then set referenceChild to node’s next sibling.
        if reference_child == Some(node) {
            reference_child = node.next_sibling()
        }

        // SPEC: 4. Insert node into parent before referenceChild.
        self.insert(node, reference_child);

        // SPEC: 5. Return node.
        node
    }
}

impl<'arena> std::fmt::Debug for Node<'arena> {
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
    Doctype {
        name: String,
        public_id: String,
        system_id: String,
    },
    Text {
        contents: RefCell<String>,
    },
    Comment {
        contents: String,
    },
    Element {
        name: QualifiedName,
        attributes: RefCell<Vec<Attribute>>,
    },
    ProcessingInstruction {
        target: String,
        contents: String,
    },
}

impl NodeData {
    pub fn tags(&self) -> (Option<String>, Option<String>) {
        let std_close = Some("\"".to_string());

        match self {
            NodeData::Document => (Some("#document".to_string()), std_close),
            NodeData::Doctype { .. } => (None, None),
            NodeData::Text { .. } => (Some("#text".to_string()), std_close),
            NodeData::Comment { .. } => (None, None),
            NodeData::Element { name, .. } => (
                Some(format!("<{}>", name.local)),
                Some(format!("</{}>", name.local)),
            ),
            NodeData::ProcessingInstruction { .. } => (None, None),
        }
    }
}

impl ToString for NodeData {
    fn to_string(&self) -> String {
        match self {
            NodeData::Document => "Document".to_string(),
            NodeData::Doctype { name, .. } => format!("DOCTYPE {name}"),
            NodeData::Text { .. } => "Text".to_string(),
            NodeData::Comment { .. } => "Comment".to_string(),
            NodeData::Element { name, .. } => {
                format!("Element({})", name.local)
            }
            NodeData::ProcessingInstruction { .. } => "ProcessingInstruction".to_string(),
        }
    }
}
