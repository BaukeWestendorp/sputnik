use std::cell::Cell;

use crate::node::Node;

pub type Arena<'arena> = &'arena typed_arena::Arena<Node<'arena>>;
pub type Ref<'arena> = &'arena Node<'arena>;
pub type Link<'arena> = Cell<Option<Ref<'arena>>>;
