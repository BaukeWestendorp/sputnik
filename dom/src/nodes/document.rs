use super::{NodeImpl, NodeType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document;

impl Document {
    pub fn new() -> Self {
        Self {}
    }

    pub fn adopt<'a, N: NodeImpl<'a>>(&self, node: &N) {
        todo!()
    }
}

impl<'a> NodeImpl<'a> for Document {
    fn node_type(&self) -> super::NodeType {
        NodeType::Document
    }

    fn node_name(&self) -> String {
        todo!()
    }

    fn node_document(&self) -> &'a Document {
        todo!()
    }
}
