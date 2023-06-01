use super::{Document, NodeImpl, NodeType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentFragment;

impl DocumentFragment {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a> NodeImpl<'a> for DocumentFragment {
    fn node_type(&self) -> super::NodeType {
        NodeType::DocumentFragment
    }

    fn node_name(&self) -> String {
        todo!()
    }

    fn node_document(&self) -> &'a Document {
        todo!()
    }
}
