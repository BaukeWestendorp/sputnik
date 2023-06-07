use dom::node::NodeRef;

use crate::tree::LayoutTree;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxType {
    Viewport,
    BlockContainer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutBox<'a> {
    node: NodeRef<'a>,

    box_type: BoxType,

    content_width: f32,
    content_height: f32,

    absolute_x: f32,
    absolute_y: f32,
}
impl<'a> LayoutBox<'a> {
    pub fn new(node: NodeRef<'a>, box_type: BoxType) -> Self {
        Self {
            node,
            box_type,
            content_width: 0.0,
            content_height: 0.0,
            absolute_x: 0.0,
            absolute_y: 0.0,
        }
    }

    pub fn node(&self) -> NodeRef<'a> {
        self.node
    }

    pub fn box_type(&self) -> BoxType {
        self.box_type
    }

    pub fn content_width(&self) -> f32 {
        self.content_width
    }

    pub fn content_height(&self) -> f32 {
        self.content_height
    }

    pub fn absolute_x(&self) -> f32 {
        self.absolute_x
    }

    pub fn absolute_y(&self) -> f32 {
        self.absolute_y
    }
}

impl<'a> LayoutBox<'a> {
    pub fn alloc(
        layout_tree: &mut LayoutTree<'a>,
        node: NodeRef<'a>,
        box_type: BoxType,
    ) -> Option<usize> {
        if !(node.is_document() || node.is_element() || node.is_text())
            || node.is_element_with_one_of_tags(&["head"])
        {
            return None;
        }

        let layout_box = LayoutBox::new(node, box_type);
        let handle = layout_tree.tree.node(layout_box);

        for child in node.child_nodes().iter() {
            if let Some(child_handle) =
                LayoutBox::alloc(layout_tree, child, BoxType::BlockContainer)
            {
                layout_tree.tree.insert(handle, child_handle);
            }
        }

        Some(handle)
    }
}
