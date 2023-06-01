use crate::infrastructure::Namespace;

use super::{Document, ElementImpl, NodeList};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<'a> {
    document: &'a Document,
}

impl<'a> NodeImpl<'a> for Node<'a> {
    fn node_type(&self) -> NodeType {
        todo!()
    }

    fn node_name(&self) -> String {
        todo!()
    }

    fn node_document(&self) -> &'a Document {
        todo!()
    }
}

pub trait NodeImpl<'a>: Eq + PartialEq {
    fn node_type(&self) -> NodeType;

    fn node_name(&self) -> String;

    fn base_uri(&self) -> String {
        todo!()
    }

    fn is_connected(&self) -> bool {
        todo!()
    }
    fn owner_document(&self) -> Option<Document> {
        todo!()
    }
    // fn get_root_node(&self, options: Option<GetRootNodeOptions>) -> &'a dyn Node {
    //     todo!()
    // }
    fn parent_node<N: NodeImpl<'a>>(&self) -> Option<&'a N> {
        todo!()
    }
    fn parent_element<E: ElementImpl<'a>>(&self) -> Option<&'a E> {
        todo!()
    }
    fn has_child_nodes(&self) -> bool {
        todo!()
    }
    fn child_nodes<N: NodeImpl<'a>>(&self) -> NodeList<'a, N> {
        todo!()
    }
    fn first_child<N: NodeImpl<'a>>(&self) -> Option<&'a N> {
        todo!()
    }
    fn last_child<N: NodeImpl<'a>>(&self) -> Option<&'a N> {
        todo!()
    }
    fn previous_sibling<N: NodeImpl<'a>>(&self) -> Option<&'a N> {
        todo!()
    }
    fn next_sibling<N: NodeImpl<'a>>(&self) -> Option<&'a N> {
        todo!()
    }

    fn node_value(&self) -> Option<String> {
        todo!()
    }
    fn text_context(&self) -> Option<String> {
        todo!()
    }
    fn normalize(&self) {
        todo!()
    }

    fn clone_node<N: NodeImpl<'a>>(&self, _deep: bool) -> &'a N {
        todo!()
    }
    fn is_equal_node<N: NodeImpl<'a>>(&self, _other_node: Option<&'a N>) -> bool {
        todo!()
    }
    fn is_same_node<N: NodeImpl<'a>>(&self, _other_node: &'a N) -> bool {
        todo!()
    }

    fn compare_document_position<N: NodeImpl<'a>>(&self, _other_node: &'a N) -> bool {
        todo!()
    }
    fn contains<N: NodeImpl<'a>>(&self, _other: &'a N) -> bool {
        todo!()
    }

    fn lookup_prefix(&self, _namespace: Option<Namespace>) -> Option<String> {
        todo!()
    }
    fn lookup_namespace_uri(&self, _prefix: Option<String>) -> Option<Namespace> {
        todo!()
    }
    fn is_default_namespace(&self, _namespace: Option<Namespace>) -> bool {
        todo!()
    }

    fn insert_before<N: NodeImpl<'a>, C: NodeImpl<'a>>(
        &self,
        _node: &'a N,
        _child: Option<&'a C>,
    ) -> bool {
        todo!()
    }
    fn append_child<N: NodeImpl<'a>>(&self, _node: &'a N) -> bool {
        todo!()
    }
    fn replace_child<N: NodeImpl<'a>>(&self, _node: &'a N, _child: Option<&'a N>) -> bool {
        todo!()
    }
    fn remove_child<N: NodeImpl<'a>>(&self, _child: &'a N) -> bool {
        todo!()
    }

    // Associated values:
    fn node_document(&self) -> &'a Document;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Element = 1,
    Attr = 2,
    Text = 3,
    CDataSection = 4,
    ProcessingInstruction = 7,
    Comment = 9,
    Document = 10,
    DocumentType = 11,
    DocumentFragment = 12,
}
