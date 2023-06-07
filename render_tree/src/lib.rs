pub mod dump;

use arena_tree::ArenaTree;
use dom::node::{Node, NodeRef, NodeType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderNode<'a> {
    Text(String),
    Element(NodeRef<'a>),
}

pub struct RenderTree<'a> {
    pub tree: ArenaTree<RenderNode<'a>>,
}

impl<'a> RenderTree<'a> {
    pub fn from(node: NodeRef<'a>) -> Self {
        let mut render_tree = Self {
            tree: ArenaTree::new(),
        };

        RenderNode::from(&mut render_tree, node);

        render_tree
    }
}

pub(crate) fn node_is_valid(node: &Node<'_>) -> bool {
    match &node.node_type {
        NodeType::Element(element) => {
            element.tag_name != "head"
                && element.tag_name != "html"
                && element.tag_name != "script"
                && element.tag_name != "meta"
                && element.tag_name != "link"
                && element.tag_name != "style"
        }
        NodeType::Text { .. } => true,
        _ => false,
    }
}

impl<'a> RenderNode<'a> {
    pub fn from(render_tree: &mut RenderTree<'a>, node: NodeRef<'a>) -> usize {
        let mut tree_node = render_tree.tree.node(RenderNode::Element(node));
        if node_is_valid(node) {
            for child in node.child_nodes().iter() {
                if node_is_valid(child) {
                    match &child.node_type {
                        NodeType::Element(_) => {
                            let new_node = RenderNode::from(render_tree, child);
                            render_tree.tree.insert(tree_node, new_node);
                        }
                        NodeType::Text { data } => {
                            let new_node = render_tree
                                .tree
                                .node(RenderNode::Text(data.borrow().clone()));
                            render_tree.tree.insert(tree_node, new_node);
                        }
                        _ => {}
                    }
                }
            }
        } else {
            for child in node.child_nodes().iter() {
                if node_is_valid(child) {
                    tree_node = RenderNode::from(render_tree, child);
                }
            }
        }

        tree_node
    }
}
