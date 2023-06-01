use crate::infrastructure::Namespace;
use crate::nodes::{Document, NodeImpl};

use super::ElementImpl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlElement {
    namespace: Option<Namespace>,
    namespace_prefix: Option<String>,
    local_name: String,
}

impl<'a> ElementImpl<'a> for HtmlElement {
    fn namespace_uri(&self) -> Option<Namespace> {
        self.namespace
    }

    fn prefix(&self) -> Option<String> {
        self.namespace_prefix.clone()
    }

    fn local_name(&self) -> String {
        self.local_name.clone()
    }

    fn tag_name(&self) -> String {
        self.html_uppercased_qualified_name()
    }
}

impl<'a> NodeImpl<'a> for HtmlElement {
    fn node_type(&self) -> crate::nodes::NodeType {
        todo!()
    }

    fn node_name(&self) -> String {
        todo!()
    }

    fn node_document(&self) -> &'a Document {
        todo!()
    }
}
