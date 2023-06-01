use crate::infrastructure::Namespace;

use super::{Element, NodeImpl, NodeType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attr<'a> {
    namespace: Option<Namespace>,
    namespace_prefix: Option<String>,
    local_name: String,
    value: String,
    element: Option<Element<'a>>,
}

impl<'a> Attr<'a> {
    pub fn namespace_uri(&self) -> &Option<Namespace> {
        &self.namespace
    }

    pub fn prefix(&self) -> &Option<String> {
        &self.namespace_prefix
    }

    pub fn local_name(&self) -> &String {
        &self.local_name
    }

    pub fn name(&self) -> String {
        self.qualified_name()
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    pub fn set_value(&mut self, value: String) {
        self.value = value;
    }

    pub fn owner_element(&self) -> Option<&Element> {
        self.element.as_ref()
    }

    pub fn specified(&self) -> bool {
        true
    }
}

impl<'a> Attr<'a> {
    pub(crate) fn qualified_name(&self) -> String {
        todo!()
    }
}

impl<'a> NodeImpl<'a> for Attr<'a> {
    fn node_type(&self) -> NodeType {
        NodeType::Attr
    }

    fn node_name(&self) -> String {
        self.qualified_name()
    }

    fn node_document(&self) -> &'a super::Document {
        todo!()
    }
}
