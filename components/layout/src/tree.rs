use crate::layout_box::{BoxType, LayoutBox};
use arena_tree::ArenaTree;
use dom::node::NodeRef;

#[derive(Debug, PartialEq)]
pub struct LayoutTree<'a> {
    pub tree: ArenaTree<LayoutBox<'a>>,
}

impl<'a> LayoutTree<'a> {
    pub fn from(document: NodeRef<'a>) -> Self {
        let mut layout_tree = Self {
            tree: ArenaTree::new(),
        };

        LayoutBox::alloc(&mut layout_tree, document, BoxType::Viewport);

        layout_tree
    }
}
