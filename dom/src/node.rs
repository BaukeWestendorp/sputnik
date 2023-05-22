use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::custom_element_definition::CustomElementDefinition;
use crate::node_list::LiveNodeList;
use crate::range::{AbstractRange, Range};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub node_type: NodeType,
    pub node_name: String,

    pub base_uri: String,

    document: RefCell<Option<Rc<Node>>>,

    parent: RefCell<Option<Rc<Node>>>,
    child_nodes: RefCell<Vec<Rc<Node>>>,
    first_child: RefCell<Option<Rc<Node>>>,
    last_child: RefCell<Option<Rc<Node>>>,
    previous_sibling: RefCell<Option<Rc<Node>>>,
    next_sibling: RefCell<Option<Rc<Node>>>,
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("<{:?} />", self.node_type))

        // write!(
        //     f,
        //     "{{ node_type: {}, children: {} }}",
        //     self.node_type,
        //     self.child_nodes()[0]
        // )
    }
}

impl Default for Node {
    fn default() -> Self {
        NULL_NODE.clone()
    }
}

macro_rules! is_node_type {
    ($fn_name:ident, $node_type:pat) => {
        pub fn $fn_name(&self) -> bool {
            if let $node_type = self.node_type {
                return true;
            }
            false
        }
    };
}

impl Node {
    pub fn new(node_type: NodeType) -> Self {
        Self {
            node_type,
            ..Default::default()
        }
    }

    pub fn document(&self) -> Ref<Option<Rc<Node>>> {
        self.document.borrow()
    }

    pub fn parent_node(&self) -> Ref<Option<Rc<Node>>> {
        self.parent.borrow()
    }

    pub fn parent_element(&self) -> Option<Rc<Node>> {
        if let Some(parent) = self.parent_node().clone() {
            if parent.is_element() {
                return self.parent_node().clone();
            }
        }
        return None;
    }

    pub fn child_nodes(&self) -> Ref<Vec<Rc<Node>>> {
        if self.child_nodes.borrow().is_empty() {
            let root = Rc::new(self.to_owned());
            let this = self.clone();
            *self.child_nodes.borrow_mut() =
                LiveNodeList::new(root, move |node| this.is_parent_of(node)).into();
        }

        self.child_nodes.borrow()
    }

    pub fn first_child(&self) -> Ref<Option<Rc<Node>>> {
        self.first_child.borrow()
    }

    pub fn last_child(&self) -> Ref<Option<Rc<Node>>> {
        self.last_child.borrow()
    }

    pub fn previous_sibling(&self) -> Ref<Option<Rc<Node>>> {
        self.previous_sibling.borrow()
    }

    pub fn next_sibling(&self) -> Ref<Option<Rc<Node>>> {
        self.next_sibling.borrow()
    }

    pub fn for_each_in_inclusive_subtree<F>(&self, mut callback: F)
    where
        F: FnMut(Rc<Node>) -> bool,
    {
        let mut visited = HashMap::<usize, bool>::new();
        let mut stack = Vec::<Rc<Node>>::new();

        if let Some(root) = self.first_child().clone() {
            stack.push(root);
        }

        while let Some(node) = stack.pop() {
            callback(Rc::new(self.clone()));

            let index = node.index();
            if !visited.contains_key(&index) {
                visited.insert(index, true);

                while let Some(last_child) = node.last_child().clone() {
                    if !visited.contains_key(&last_child.index()) {
                        stack.push(last_child);
                    }
                }
            }
        }
    }

    pub fn is_parent_of(&self, node: Rc<Node>) -> bool {
        let mut current = self.first_child().clone();
        while let Some(child) = current.clone() {
            if node == child {
                return true;
            }
            current = child.next_sibling().clone();
        }
        false
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-append
    pub fn append_child(&self, node: Rc<Node>) -> Rc<Node> {
        // SPEC: To append a node to a parent, pre-insert node into parent before null.
        self.pre_insert(node, None)
    }

    pub fn has_child_nodes() -> bool {
        todo!();
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-pre-insert
    fn pre_insert(&self, node: Rc<Node>, child: Option<Rc<Node>>) -> Rc<Node> {
        // SPEC: 1. Ensure pre-insertion validity of node into parent before child.
        // FIXME Implement

        // SPEC: 2. Let referenceChild be child.
        let mut reference_child = child;

        // SPEC: 3. If referenceChild is node, then set referenceChild to node’s next sibling.
        if reference_child == Some(node.clone()) {
            if let Some(next_sibling) = node.clone().next_sibling().clone() {
                reference_child = Some(next_sibling);
            }
        }

        // SPEC: 4. Insert node into parent before referenceChild.
        self.insert_before(node.clone(), reference_child, false);

        // SPEC: 5. Return node
        node
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-tree-index
    fn index(&self) -> usize {
        // SPEC: The index of an object is its number of preceding siblings, or 0 if it has none.
        let mut index = 0;
        let mut previous_sibling = self.previous_sibling().clone();
        while let Some(_) = previous_sibling {
            index += 1;
            previous_sibling = self.previous_sibling().clone();
        }
        index
    }

    // SPECLINK:
    #[allow(dead_code)]
    fn is_inclusive_descendant_of(&self, _node: Rc<Node>) -> bool {
        todo!();
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-remove
    #[allow(dead_code)]
    fn remove(&self, node: Rc<Node>) {
        // SPEC: 1. Let parent be node’s parent.
        let parent = node.parent_node().clone();

        // SPEC: 2. Assert: parent is non-null.
        if parent.is_none() {
            return;
        }
        let parent = parent.unwrap().clone();

        // SPEC: 3. Let index be node’s index.
        let index = node.index();

        // SPEC: 4. For each live range whose start node is an inclusive descendant of node, set its start to (parent, index).
        for mut range in Range::live_ranges() {
            if range
                .start_container()
                .is_inclusive_descendant_of(Rc::new(self.clone()))
            {
                range.set_start(parent.clone(), index);
            }
        }

        // SPEC: 5. For each live range whose end node is an inclusive descendant of node, set its end to (parent, index).

        // SPEC: 6 For each live range whose start node is parent and start offset is greater than index, decrease its start offset by 1.
        // SPEC: 7. For each live range whose end node is parent and end offset is greater than index, decrease its end offset by 1.
        // SPEC: 8. For each NodeIterator object iterator whose root’s node document is node’s node document, run the NodeIterator pre-removing steps given node and iterator.
        // SPEC: 9. Let oldPreviousSibling be node’s previous sibling.
        // SPEC: 10. Let oldNextSibling be node’s next sibling.
        // SPEC: 11. Remove node from its parent’s children.
        // SPEC: 12. If node is assigned, then run assign slottables for node’s assigned slot.
        // SPEC: 13. If parent’s root is a shadow root, and parent is a slot whose assigned nodes is the empty list, then run signal a slot change for parent.
        // SPEC: 14. If node has an inclusive descendant that is a slot, then:
        // SPEC: 14.1. Run assign slottables for a tree with parent’s root.
        // SPEC: 14.2. Run assign slottables for a tree with node.
        // SPEC: 15. Run the removing steps with node and parent.
        // SPEC: 16. Let isParentConnected be parent’s connected.
        // SPEC: 17. If node is custom and isParentConnected is true, then enqueue a custom element callback reaction with node, callback name "disconnectedCallback", and an empty argument list.
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-adopt
    fn adopt(&self, _node: Rc<Node>) {
        // SPEC: 1. Let oldDocument be node’s node document.
        // let old_document = node.document();
        // SPEC: 2. If node’s parent is non-null, then remove node.
        // if let Some(parent) = node.parent_element().clone() {
        //     self.document
        //         .borrow_mut()
        //         .clone()
        //         .unwrap()
        //         .remove(node.clone());
        // }

        todo!();
        // SPEC: 3. If document is not oldDocument, then:
        // SPEC: 3.1. For each inclusiveDescendant in node’s shadow-including inclusive descendants:
        // SPEC: 3.1.1 Set inclusiveDescendant’s node document to document.
        // SPEC: 3.1.2. If inclusiveDescendant is an element, then set the node document of each attribute in inclusiveDescendant’s attribute list to document.
        // SPEC: 3.2. For each inclusiveDescendant in node’s shadow-including inclusive descendants that is custom, enqueue a custom element callback reaction with inclusiveDescendant, callback name "adoptedCallback", and an argument list containing oldDocument and document.
        // SPEC: 3.3. For each inclusiveDescendant in node’s shadow-including inclusive descendants, in shadow-including tree order, run the adopting steps with inclusiveDescendant and oldDocument.
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-insert
    fn insert_before(&self, node: Rc<Node>, child: Option<Rc<Node>>, _suppress_observers: bool) {
        let nodes = match node.node_type {
            // SPEC: 1. Let nodes be node’s children, if node is a DocumentFragment node;
            NodeType::DocumentFragment { .. } => node.child_nodes.clone(),
            // FIXME: Should node.child_nodes be node.child_nodes() ?
            // SPEC: otherwise « node ».
            _ => RefCell::new(vec![node.clone()]),
        };

        // SPEC: 2. Let count be nodes’s size.
        //       3. If count is 0, then return.
        if nodes.borrow().is_empty() {
            return;
        }

        // SPEC: 4. If node is a DocumentFragment node, then:
        if let NodeType::DocumentFragment { .. } = node.clone().node_type {
            // SPEC: 4.1. Remove its children with the suppress observers flag set.
            // SPEC: 4.2. Queue a tree mutation record for node with « », nodes, null, and null.
            todo!();
        }

        // SPEC: 5. If child is non-null, then:
        if let Some(_child) = child {
            // SPEC: 5.1. For each live range whose start node is parent and start offset is greater than child’s index, increase its start offset by count.
            // SPEC: 5.2 For each live range whose end node is parent and end offset is greater than child’s index, increase its end offset by count.
            todo!();
        }

        // SPEC: 6. Let previousSibling be child’s previous sibling or parent’s last child if child is null.
        // FIXME Implement

        // SPEC: 7. For each node in nodes, in tree order:
        for node_to_insert in nodes.borrow().clone() {
            // SPEC: 7.1. Adopt node into parent’s node document.
            if let Some(document) = self.document.borrow_mut().clone() {
                document.adopt(Rc::new(self.clone()));
            }

            // SPEC: 7.2. If child is null, then append node to parent’s children.
            //       7.3. Otherwise, insert node into parent’s children before child’s index.
            if let Some(child) = child.clone() {
                self.insert_before_impl(node_to_insert, child);
            } else {
                self.append_child_impl(node_to_insert);
            }

            // FIXME Implement SPEC: 7.4. If parent is a shadow host whose shadow root’s slot assignment is "named" and node is a slottable, then assign a slot for node.
            // FIXME Implement SPEC: 7.5. If parent’s root is a shadow root, and parent is a slot whose assigned nodes is the empty list, then run signal a slot change for parent.
            // FIXME Implement SPEC: 7.6. Run assign slottables for a tree with node’s root.

            // FIXME Implement SPEC: 7.7. For each shadow-including inclusive descendant inclusiveDescendant of node, in shadow-including tree order:
            // FIXME Implement SPEC: 7.7.1. Run the insertion steps with inclusiveDescendant.
            // FIXME Implement SPEC: 7.7.2. If inclusiveDescendant is connected, then:
            // FIXME Implement SPEC: 7.7.2.1. If inclusiveDescendant is custom, then enqueue a custom element callback reaction with inclusiveDescendant, callback name "connectedCallback", and an empty argument list.
            // FIXME Implement SPEC: 7.7.2.1. Otherwise, try to upgrade inclusiveDescendant.

            // FIXME Implement SPEC: 8. If suppress observers flag is unset, then queue a tree mutation record for parent with nodes, « », previousSibling, and child.

            // SPEC: 9. Run the children changed steps for parent.
            self.children_changed();
        }
    }

    is_node_type!(is_element, NodeType::Element(_));
    is_node_type!(is_attr, NodeType::Attr(_));
    is_node_type!(is_text, NodeType::Text(_));
    is_node_type!(is_cdata_section, NodeType::CDATASection());
    is_node_type!(is_processing_instruction, NodeType::ProcessingInstruction());
    is_node_type!(is_comment, NodeType::Comment(_));
    is_node_type!(is_document, NodeType::Document());
    is_node_type!(is_document_type, NodeType::DocumentType(_));
    is_node_type!(is_document_fragment, NodeType::DocumentFragment());
    is_node_type!(is_null, NodeType::Null);

    fn append_child_impl(&self, node: Rc<Node>) {
        assert!(node.parent_node().is_none());

        if !self.is_child_allowed(node.clone()) {
            return;
        }

        if self.last_child().is_some() {
            *self
                .last_child
                .borrow()
                .clone()
                .unwrap()
                .next_sibling
                .borrow_mut() = Some(node.clone());
        }

        *node.previous_sibling.borrow_mut() = self.last_child().clone();

        *node.parent.borrow_mut() = Some(Rc::new(self.clone()));

        *self.last_child.borrow_mut() = Some(node.clone());

        if self.first_child.borrow().is_none() {
            *self.first_child.borrow_mut() = self.last_child().clone()
        }
    }

    fn insert_before_impl(&self, _node: Rc<Node>, _child: Rc<Node>) {
        todo!()
    }

    fn is_child_allowed(&self, _node: Rc<Node>) -> bool {
        true
    }

    fn children_changed(&self) {}
}

const NULL_NODE: Node = Node {
    node_type: NodeType::Null {},
    node_name: String::new(),
    base_uri: String::new(),
    parent: RefCell::new(None),
    child_nodes: RefCell::new(Vec::new()),
    first_child: RefCell::new(None),
    last_child: RefCell::new(None),
    previous_sibling: RefCell::new(None),
    next_sibling: RefCell::new(None),
    document: RefCell::new(None),
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociatedValues {
    pub namespace: Option<String>,
    pub namespace_prefix: Option<String>,
    pub local_name: String,
    pub custom_element_state: CustomElementState,
    pub custom_element_definition: Option<CustomElementDefinition>,
    pub is: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomElementState {
    Undefined,
    Failed,
    Uncustomized,
    Precustomized,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Element(Element),
    Attr(Attr),
    Text(Text),
    CDATASection(),
    ProcessingInstruction(),
    Comment(Comment),
    Document(),
    DocumentType(DocumentType),
    DocumentFragment(),
    Null,
}

impl Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            NodeType::Element(_) => "Element",
            NodeType::Attr(_) => "Attr",
            NodeType::Text(_) => "Text",
            NodeType::CDATASection() => "CDATASection",
            NodeType::ProcessingInstruction() => "ProcessingInstruction",
            NodeType::Comment(_) => "Comment",
            NodeType::Document() => "Document",
            NodeType::DocumentType(_) => "DocumentType",
            NodeType::DocumentFragment() => "DocumentFragment",
            NodeType::Null => "Null",
        };
        write!(f, "{name}")
    }
}

// SPECLINK: https://dom.spec.whatwg.org/#interface-element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub associated_values: AssociatedValues,
    pub namespace_uri: Option<String>,
    pub prefix: Option<String>,
    pub local_name: String,
    pub tag_name: String,
    pub attributes: HashMap<String, NodeType>, // FIXME: HashMap should be NamedNodeMap instead
}

impl Element {
    pub fn new(
        associated_values: AssociatedValues,
        local_name: String,
        namespace_uri: Option<String>,
        prefix: Option<String>,
        tag_name: String,
    ) -> Self {
        Self {
            attributes: HashMap::new(),
            associated_values,
            local_name,
            namespace_uri,
            prefix,
            tag_name,
        }
    }
}

// SPECLINK: https://dom.spec.whatwg.org/#interface-attr
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attr {
    pub value: String,
}

impl Attr {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

// SPECLINK: https://dom.spec.whatwg.org/#interface-text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    pub data: String,
}

impl Text {
    pub fn new(data: &str) -> Self {
        Self {
            data: data.to_string(),
        }
    }
}

// SPECLINK: https://dom.spec.whatwg.org/#interface-comment
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub data: String,
}

impl Comment {
    pub fn new(data: &str) -> Self {
        Self {
            data: data.to_string(),
        }
    }
}

// SPECLINK: https://dom.spec.whatwg.org/#interface-documenttype
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentType {
    pub name: String,
    pub public_id: String,
    pub system_id: String,
}

impl DocumentType {
    pub fn new(name: String, public_id: String, system_id: String) -> Self {
        Self {
            name,
            public_id,
            system_id,
        }
    }
}
