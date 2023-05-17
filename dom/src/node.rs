use std::collections::HashMap;
use std::rc::Rc;

use crate::custom_element_definition::CustomElementDefinition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub node_type: NodeType,
    pub node_name: String,

    pub base_uri: String,

    pub is_connected: bool,
    pub owner_document: Option<Rc<Node>>,
    pub parent_node: Option<Rc<Node>>,
    pub parent_element: Option<Rc<Node>>,
    pub child_nodes: Vec<Rc<Node>>,
    pub first_child: Option<Rc<Node>>,
    pub last_child: Option<Rc<Node>>,
    pub previous_sibling: Option<Rc<Node>>,
    pub next_sibling: Option<Rc<Node>>,

    pub document: Option<Rc<Node>>,
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
            node.next_sibling.clone_into(&mut reference_child);
        }

        // SPEC: 4. Insert node into parent before referenceChild.
        self.insert_before(node.clone(), reference_child, false);

        // SPEC: 5. Return node
        node
    }

    // SPECLINK: https://dom.spec.whatwg.org/#concept-node-insert
    fn insert_before(&self, node: Rc<Node>, child: Option<Rc<Node>>, _suppress_observers: bool) {
        let nodes = match node.node_type {
            // SPEC: 1. Let nodes be node’s children, if node is a DocumentFragment node;
            NodeType::DocumentFragment { .. } => node.child_nodes.clone(),
            // SPEC: otherwise « node ».
            _ => vec![node.clone()],
        };

        // SPEC: 2. Let count be nodes’s size.
        //       3. If count is 0, then return.
        if nodes.is_empty() {
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
}

const NULL_NODE: Node = Node {
    node_type: NodeType::Null {},
    node_name: String::new(),
    base_uri: String::new(),
    is_connected: false,
    owner_document: None,
    parent_node: None,
    parent_element: None,
    child_nodes: Vec::new(),
    first_child: None,
    last_child: None,
    previous_sibling: None,
    next_sibling: None,
    document: None,
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
