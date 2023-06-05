use std::cell::RefCell;

use crate::namespace::Namespace;
use crate::types::NodeRef;
use crate::Parser;

pub use node::*;

pub(crate) mod mutation_algorithms;
pub mod node;

impl<'a> Parser<'a> {
    // https://dom.spec.whatwg.org/#concept-create-element
    pub(crate) fn create_element(
        &'a self,
        document: NodeRef<'a>,
        local_name: &String,
        namespace: Namespace,
        _prefix: Option<&String>,
        _is: Option<&String>,
        _synchronous_custom_elements: bool,
    ) -> NodeRef<'a> {
        // FIXME: This does not implement any spec functionality yet!
        self.allocate_node(Node::new(
            Some(document),
            NodeType::Element {
                tag_name: local_name.to_owned(),
                namespace: Some(namespace),
                attributes: RefCell::new(vec![]),
            },
        ))
    }
}
