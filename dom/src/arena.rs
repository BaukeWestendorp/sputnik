use std::cell::Cell;

use crate::node::Node;

pub type Arena<'a> = &'a typed_arena::Arena<Node<'a>>;
pub type Ref<'a> = &'a Node<'a>;
pub type Link<'a> = Cell<Option<Ref<'a>>>;
