use std::cell::Cell;

use crate::node::Node;

pub type Arena<'a> = &'a typed_arena::Arena<Node<'a>>;
pub type NodeRef<'a> = &'a Node<'a>;
pub type NodeLink<'a> = Cell<Option<NodeRef<'a>>>;
