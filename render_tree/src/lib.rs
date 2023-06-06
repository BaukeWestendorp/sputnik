pub mod dump;

use dom::node::{Node, NodeType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderObject<'a> {
    Text(String),
    Element {
        element: Node<'a>,
        children: Vec<RenderObject<'a>>,
        // FIXME: Style
    },
}

fn node_is_valid<'a>(node: &Node<'a>) -> bool {
    match &node.node_type {
        NodeType::Element(element) => {
            element.tag_name != "head"
                && element.tag_name != "script"
                && element.tag_name != "meta"
                && element.tag_name != "link"
                && element.tag_name != "style"
        }
        NodeType::Text { .. } => true,
        _ => false,
    }
}

fn next_valid_render_tree_descendant<'a>(node: &Node<'a>) -> Option<&'a Node<'a>> {
    let mut valid_child = None;
    for child in node.child_nodes().iter() {
        if node_is_valid(child) {
            valid_child = Some(child.clone());
            break;
        }

        if let Some(next_valid_child) = next_valid_render_tree_descendant(child) {
            valid_child = Some(next_valid_child);
            break;
        }
    }
    valid_child
}

impl<'a> From<Node<'a>> for RenderObject<'a> {
    fn from(node: Node<'a>) -> Self {
        let mut render_object = RenderObject::Element {
            element: node.clone(),
            children: vec![],
        };

        if node_is_valid(&node) {
            match &node.node_type {
                NodeType::Element { .. } => {
                    for child in node.child_nodes().iter() {
                        if node_is_valid(child) {
                            render_object.append_child_if_possible(RenderObject::from(
                                <&Node<'_>>::clone(child).clone(),
                            ));
                        }
                    }
                }
                NodeType::Text { data } => {
                    render_object
                        .append_child_if_possible(RenderObject::Text(data.borrow().clone()));
                }
                _ => {}
            }
        } else {
            for child in node.child_nodes().iter() {
                if node_is_valid(child) {
                    render_object = RenderObject::from(child.clone().clone());
                }
            }
        }

        render_object.clone()
    }
}

impl<'a> RenderObject<'a> {
    fn append_child_if_possible(&mut self, child: RenderObject<'a>) {
        if let RenderObject::Element { children, .. } = self {
            children.push(child)
        }
    }
}
