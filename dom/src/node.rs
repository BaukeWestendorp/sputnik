use std::cell::{Cell, RefCell};

use crate::arena::{Link, Ref};
use crate::mutation_observer::RegisteredObserver;
use crate::mutation_record::MutationRecord;
use crate::{Attribute, QualifiedName};

/// A HTML Node.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct Node<'arena> {
    document: Link<'arena>,
    parent: Link<'arena>,
    children: RefCell<Vec<Ref<'arena>>>,
    registered_observer_list: RefCell<Vec<RegisteredObserver>>,
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
            registered_observer_list: RefCell::new(Vec::new()),
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

    pub fn registered_observer_list(&'arena self) -> std::cell::Ref<Vec<RegisteredObserver>> {
        self.registered_observer_list.borrow()
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

    fn internal_dump(&'arena self, prefix: &str) {
        let name = self.data.to_string();
        println!("{}<{}>", prefix, name);
        for child in self.children().iter() {
            child.internal_dump("  ");
        }
        println!("{}</{}>", prefix, name);
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

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-pre-insert
    fn pre_insert(&'arena self, node: &'arena Self, before: Option<&'arena Self>) -> &'arena Self {
        // SPEC: 1. Ensure pre-insertion validity of node into parent before child.
        // FIXME: Implement

        // SPEC: 2. Let referenceChild be child.
        let mut reference_child = before;

        // SPEC: 3. If referenceChild is node, then set referenceChild to node’s next sibling.
        if reference_child == Some(node) {
            reference_child = node.next_sibling()
        }

        // SPEC: 4. Insert node into parent before referenceChild.
        self.insert(node, reference_child);

        // SPEC: 5. Return node.
        node
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-insert
    fn insert(&'arena self, node: &'arena Self, before: Option<&'arena Self>) {
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
        if let Some(_before) = before {
            todo!()
        }

        // SPEC: 6. Let previousSibling be child’s previous sibling or parent’s last child if child is null.
        let previous_sibling = match before {
            Some(before) => before.previous_sibling(),
            None => self.last_child(),
        };

        // SPEC: 7. For each node in nodes, in tree order:
        for node in nodes.iter() {
            // SPEC: 7.1. Adopt node into parent’s node document.
            self.document().adopt(node);

            if let Some(before) = before {
                // SPEC: 7.3. Otherwise, insert node into parent’s children before child’s index.
                self.children.borrow_mut().insert(before.index(), node);
            } else {
                // SPEC: 7.2. If child is null, then append node to parent’s children.
                self.children.borrow_mut().push(node);
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
        MutationRecord::queue_tree_mutation_record(
            self,
            nodes,
            Vec::new(),
            previous_sibling,
            before,
        )

        // SPEC: 9. Run the children changed steps for parent.
        // FIXME: Implement
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

impl ToString for NodeData {
    fn to_string(&self) -> String {
        match self {
            NodeData::Document => "Document".to_string(),
            NodeData::Doctype { name, .. } => format!("DOCTYPE {name}"),
            NodeData::Text { .. } => "Text".to_string(),
            NodeData::Comment { .. } => "Comment".to_string(),
            NodeData::Element { name, .. } => {
                format!("{}", name.local)
            }
            NodeData::ProcessingInstruction { .. } => "ProcessingInstruction".to_string(),
        }
    }
}
