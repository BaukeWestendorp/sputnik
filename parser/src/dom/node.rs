use std::cell::{Cell, Ref, RefCell};

use tokenizer::Attribute;

use crate::namespace::Namespace;
use crate::types::{NodeLink, NodeRef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Element {
        // FIXME: This does not really follow the spec.
        tag_name: String,
        namespace: Option<Namespace>,
        attributes: RefCell<Vec<Attribute>>,
    },
    Attr,
    Text {
        data: RefCell<String>,
    },
    CDataSection,
    ProcessingInstruction,
    Comment,
    Document,
    DocumentType {
        name: String,
        public_identifier: String,
        system_identifier: String,
    },
    DocumentFragment,
}

#[derive(Debug, Clone, Eq)]
pub struct Node<'a> {
    pub node_type: NodeType,

    pub(super) parent: NodeLink<'a>,
    pub(super) first_child: NodeLink<'a>,
    pub(super) last_child: NodeLink<'a>,
    pub(super) next_sibling: NodeLink<'a>,
    pub(super) previous_sibling: NodeLink<'a>,
    pub(super) children: RefCell<Vec<NodeRef<'a>>>,

    pub(super) node_document: NodeLink<'a>,
}

// IDL
// https://dom.spec.whatwg.org/#interface-node
impl<'a> Node<'a> {
    // https://dom.spec.whatwg.org/#dom-node-nodename
    pub fn node_name(&'a self) -> String {
        match &self.node_type {
            NodeType::Element { tag_name, .. } => {
                // FIXME: Implement propperly
                tag_name.to_ascii_uppercase()
            }
            NodeType::Attr => todo!(),
            NodeType::Text { .. } => "#text".to_string(),
            NodeType::CDataSection => "#cdata-section".to_string(),
            NodeType::ProcessingInstruction => todo!(),
            NodeType::Comment => "#comment".to_string(),
            NodeType::Document => "#document".to_string(),
            NodeType::DocumentType { name, .. } => name.to_string(),
            NodeType::DocumentFragment => "#document-fragment".to_string(),
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-parentnode
    pub fn parent_node(&'a self) -> Option<NodeRef<'a>> {
        self.parent.get()
    }

    // https://dom.spec.whatwg.org/#dom-node-parentelement
    pub fn parent_element(&'a self) -> Option<NodeRef<'a>> {
        match self.parent.get() {
            Some(parent) if parent.is_element() => Some(parent),
            _ => None,
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-childnodes
    // FIXME: Should return a NodeList.
    pub fn child_nodes(&'a self) -> Ref<Vec<NodeRef<'a>>> {
        self.children.borrow()
    }

    // https://dom.spec.whatwg.org/#dom-node-firstchild
    pub fn first_child(&'a self) -> Option<NodeRef<'a>> {
        self.first_child.get()
    }

    // https://dom.spec.whatwg.org/#dom-node-lastchild
    pub fn last_child(&'a self) -> Option<NodeRef<'a>> {
        self.last_child.get()
    }

    // https://dom.spec.whatwg.org/#dom-node-previoussibling
    pub fn previous_sibling(&'a self) -> Option<NodeRef<'a>> {
        self.previous_sibling.get()
    }

    // https://dom.spec.whatwg.org/#dom-node-nextsibling
    pub fn next_sibling(&'a self) -> Option<NodeRef<'a>> {
        self.next_sibling.get()
    }

    // https://dom.spec.whatwg.org/#dom-node-appendchild
    pub fn append_child(&'a self, node: NodeRef<'a>) {
        Node::append(node, self, false)
    }
}

// Concepts
impl<'a> Node<'a> {
    // https://dom.spec.whatwg.org/#concept-node-document
    pub fn node_document(&'a self) -> NodeRef<'a> {
        match self.node_document.get() {
            Some(node_document) => node_document,
            None => self,
        }
    }

    // https://dom.spec.whatwg.org/#concept-tree-index
    pub fn index(&'a self) -> usize {
        let mut index = 0;
        let mut current = self.previous_sibling();
        while let Some(node) = current {
            index += 1;
            current = node.next_sibling()
        }
        index
    }

    // https://dom.spec.whatwg.org/#concept-node-adopt
    pub fn adopt(&'a self, node: NodeRef<'a>) {
        if !self.is_document() {
            panic!("only the Document Node should adopt nodes");
        }

        // 1. Let oldDocument be node’s node document.
        let old_document = node.node_document();

        // 2. If node’s parent is non-null, then remove node.
        if node.parent_node().is_some() {
            Node::remove(node, false);
        }

        // 3. If document is not oldDocument, then:
        if self.node_document() == old_document {
            // 3.1. For each inclusiveDescendant in node’s shadow-including inclusive descendants:
            for inclusive_descendant in node.shadow_including_inclusive_descendants().iter() {
                // 3.1.1. Set inclusiveDescendant’s node document to document.
                inclusive_descendant
                    .node_document
                    .set(Some(self.node_document()));

                // 3.1.2. If inclusiveDescendant is an element, then set the node document of each attribute in inclusiveDescendant’s attribute list to document.
                if inclusive_descendant.is_element() {
                    todo!()
                }
            }

            // FIXME: 3.2. For each inclusiveDescendant in node’s shadow-including inclusive descendants that is custom, enqueue a custom element callback reaction with inclusiveDescendant, callback name "adoptedCallback", and an argument list containing oldDocument and document.
            // FIXME: 3.3. For each inclusiveDescendant in node’s shadow-including inclusive descendants, in shadow-including tree order, run the adopting steps with inclusiveDescendant and oldDocument.
        }
    }

    // https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-descendant
    pub fn shadow_including_inclusive_descendants(&'a self) -> Vec<NodeRef<'a>> {
        // FIXME: Implement properly.
        self.child_nodes().to_owned()
    }
}

macro_rules! is_node_type {
    ($fn_name:ident, $node_type:pat) => {
        pub fn $fn_name(&self) -> bool {
            matches!(self.node_type, $node_type)
        }
    };
}

// Helpers
impl<'a> Node<'a> {
    pub fn new(document: Option<NodeRef<'a>>, node_type: NodeType) -> Self {
        Self {
            parent: Cell::new(None),
            next_sibling: Cell::new(None),
            previous_sibling: Cell::new(None),
            first_child: Cell::new(None),
            last_child: Cell::new(None),
            children: RefCell::new(vec![]),
            node_type,
            node_document: Cell::new(document),
        }
    }

    is_node_type!(is_element, NodeType::Element { .. });
    is_node_type!(is_attr, NodeType::Attr);
    is_node_type!(is_text, NodeType::Text { .. });
    is_node_type!(is_cdata_section, NodeType::CDataSection);
    is_node_type!(is_processing_instruction, NodeType::ProcessingInstruction);
    is_node_type!(is_comment, NodeType::Comment);
    is_node_type!(is_document, NodeType::Document);
    is_node_type!(is_document_type, NodeType::DocumentType { .. });
    is_node_type!(is_document_fragment, NodeType::DocumentFragment);

    pub fn element_tag_name(&self) -> Option<String> {
        match &self.node_type {
            NodeType::Element { tag_name, .. } => Some(tag_name.to_string()),
            _ => None,
        }
    }

    pub fn is_element_with_one_of_tags(&self, tags: &[&str]) -> bool {
        if let NodeType::Element { tag_name, .. } = &self.node_type {
            return tags.contains(&tag_name.as_str());
        }
        false
    }

    pub fn is_element_with_tag(&self, tag: &str) -> bool {
        if let NodeType::Element { tag_name, .. } = &self.node_type {
            return tag_name == tag;
        }
        false
    }

    pub fn is_element_with_namespace(&self, namespace: Namespace) -> bool {
        if let NodeType::Element { namespace: ns, .. } = &self.node_type {
            return Some(namespace) == *ns;
        }
        false
    }

    pub fn dump(&'a self, settings: NodeDumpSettings) {
        self.internal_dump("", &settings);
    }

    fn internal_dump(&'a self, indentation: &str, settings: &NodeDumpSettings) {
        let indent = "  ";

        macro_rules! color {
            ($color:literal) => {
                if settings.color {
                    $color
                } else {
                    ""
                }
            };
        }

        let yellow = color!("\x1b[33m");
        let blue = color!("\x1b[34m");
        let cyan = color!("\x1b[36m");
        let white = color!("\x1b[37m");
        let reset = color!("\x1b[0m");
        let green = color!("\x1b[32m");
        let gray = color!("\x1b[90m");

        let opening = match &self.node_type {
            NodeType::DocumentType { name, .. } => {
                format!("{yellow}DOCTYPE {white}{}{reset}", name)
            }
            NodeType::Text { data } => {
                format!("{gray}{}: \"{white}{}{gray}\"{reset}", self.node_name(), {
                    let data = data.borrow().clone();
                    match settings.trim_text {
                        true => data.trim().to_string(),
                        false => data.clone(),
                    }
                })
            }
            NodeType::Element { attributes, .. } => {
                let mut attr_string = String::new();
                for attr in attributes.borrow().iter() {
                    attr_string.push_str(&format!(
                        "{cyan}{}{gray}{blue}={gray}\"{green}{}{gray}\" ",
                        attr.name, attr.value
                    ));
                }
                format!("{yellow}{} {}{reset}", self.node_name(), attr_string)
            }
            _ => format!("{gray}{}{reset}", self.node_name()),
        };
        println!("{indentation}{}", opening);
        for child in self.child_nodes().iter() {
            let mut indentation = indentation.to_string();
            indentation.push_str(indent);
            child.internal_dump(&indentation, settings);
        }
        if let Some(closing_marker) = settings.closing_marker {
            println!("{indentation}{closing_marker}");
        }
    }
}

// https://dom.spec.whatwg.org/#concept-node-equals
impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Self) -> bool {
        // A and B implement the same interfaces.
        // The following are equal, switching on the interface A implements:
        // If A is an element, each attribute in its attribute list has an attribute that equals an attribute in B’s attribute list.
        // A and B have the same number of children.
        // Each child of A equals the child of B at the identical index.

        self.node_type == other.node_type
            && match (self.node_type.clone(), other.node_type.clone()) {
                (
                    NodeType::DocumentType {
                        name: name_a,
                        public_identifier: pub_id_a,
                        system_identifier: sys_id_a,
                    },
                    NodeType::DocumentType {
                        name: name_b,
                        public_identifier: pub_id_b,
                        system_identifier: sys_id_b,
                    },
                ) => name_a == name_b && pub_id_a == pub_id_b && sys_id_a == sys_id_b,
                (
                    NodeType::Element {
                        tag_name: tag_name_a,
                        namespace: namespace_a,
                        attributes: attributes_a,
                    },
                    NodeType::Element {
                        tag_name: tag_name_b,
                        namespace: namespace_b,
                        attributes: attributes_b,
                    },
                ) => {
                    tag_name_a == tag_name_b
                        && namespace_a == namespace_b
                        && attributes_a.borrow().len() == attributes_b.borrow().len()
                }
                (NodeType::Attr, NodeType::Attr) => todo!(),
                (NodeType::ProcessingInstruction, NodeType::ProcessingInstruction) => todo!(),
                (NodeType::Text { data: data_a }, NodeType::Text { data: data_b }) => {
                    data_a == data_b
                }
                (NodeType::Comment, NodeType::Comment) => todo!(),
                _ => true,
            }
            && match &self.node_type {
                NodeType::Element { attributes, .. } => {
                    if let NodeType::Element {
                        attributes: other_attributes,
                        ..
                    } = &other.node_type
                    {
                        attributes.borrow().iter().enumerate().all(|(i, attr)| {
                            let other_attributes = other_attributes.borrow();
                            Some(attr) == other_attributes.get(i)
                        })
                    } else {
                        false
                    }
                }
                _ => true,
            }
            && self.children.borrow().len() == other.children.borrow().len()
            && self.children.borrow().iter().all(|child| {
                let other_children = other.children.borrow();
                let other_child = other_children.get(child.index());
                child == other_child.unwrap()
            })
    }
}

pub struct NodeDumpSettings {
    closing_marker: Option<&'static str>,
    color: bool,
    trim_text: bool,
}

impl Default for NodeDumpSettings {
    fn default() -> Self {
        Self {
            closing_marker: None,
            color: true,
            trim_text: true,
        }
    }
}
