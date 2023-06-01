pub mod html;

pub use html::*;

use crate::infrastructure::Namespace;

use super::{Attr, Document, NodeImpl, NodeType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element<'a> {
    document: &'a Document,

    namespace: Option<Namespace>,
    namespace_prefix: Option<String>,
    local_name: String,
    custom_element_state: (),
    custom_element_definition: (),
    is: Option<String>,

    attribute_list: Vec<Attr<'a>>,
}

impl<'a> ElementImpl<'a> for Element<'a> {
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

impl<'a> NodeImpl<'a> for Element<'a> {
    fn node_type(&self) -> super::NodeType {
        NodeType::Element
    }

    fn node_name(&self) -> String {
        todo!()
    }

    fn node_document(&self) -> &'a Document {
        todo!()
    }
}

pub trait ElementImpl<'a>: NodeImpl<'a> {
    // IDL
    fn namespace_uri(&self) -> Option<Namespace>;
    fn prefix(&self) -> Option<String>;
    fn local_name(&self) -> String;
    fn tag_name(&self) -> String;

    // Helpers
    fn create(
        document: &'a Document,
        local_name: &'a str,
        namespace: Option<Namespace>,
        prefix: Option<String>,
        is: Option<String>,
    ) -> Element<'a> {
        // FIXME: Make this spec compliant

        // SPEC: Set result to a new element that implements interface,
        //       with no attributes,
        //       namespace set to namespace,
        //       namespace prefix set to prefix,
        //       local name set to localName,
        //       custom element state set to "uncustomized",
        //       custom element definition set to null,
        //       is value set to is,
        //       and node document set to document.

        Element {
            document,
            namespace,
            namespace_prefix: prefix,
            local_name: local_name.to_string(),
            custom_element_state: (),
            custom_element_definition: (),
            is,
            attribute_list: vec![],
        }
    }

    fn html_uppercased_qualified_name(&self) -> String {
        let mut qualified_name = self.qualified_name();
        if self.namespace_uri() == Some(Namespace::Html)
            && self.node_document().lookup_namespace_uri(None) == Some(Namespace::Html)
        {
            qualified_name = qualified_name.to_ascii_uppercase();
        }

        qualified_name
    }

    fn qualified_name(&self) -> String {
        if let Some(namespace_prefix) = self.prefix() {
            return format!("{}:{}", namespace_prefix, self.local_name());
        }
        self.local_name()
    }
}
