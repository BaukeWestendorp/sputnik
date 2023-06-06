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

impl<'a> RenderObject<'a> {
    pub fn element(&self) -> Option<&Node<'a>> {
        match self {
            RenderObject::Text(_) => None,
            RenderObject::Element { element, .. } => Some(element),
        }
    }

    pub fn children(&self) -> Vec<RenderObject<'a>> {
        match self {
            RenderObject::Text(_) => vec![],
            RenderObject::Element { children, .. } => children.clone(),
        }
    }

    pub(crate) fn append_child_if_possible(&mut self, child: RenderObject<'a>) {
        if let RenderObject::Element { children, .. } = self {
            children.push(child)
        }
    }
}

pub(crate) fn node_is_valid(node: &Node<'_>) -> bool {
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

impl<'a> From<Node<'a>> for RenderObject<'a> {
    fn from(node: Node<'a>) -> Self {
        let mut render_object = RenderObject::Element {
            element: node.clone(),
            children: vec![],
        };

        // eprintln!("Node {:?}", node.node_type);

        if node_is_valid(&node) {
            for child in node.child_nodes().iter() {
                if node_is_valid(child) {
                    match &child.node_type {
                        NodeType::Element(_) => {
                            render_object.append_child_if_possible(RenderObject::from(
                                <&Node<'_>>::clone(child).clone(),
                            ));
                        }
                        NodeType::Text { data } => {
                            render_object.append_child_if_possible(RenderObject::Text(
                                data.borrow().clone(),
                            ));
                        }
                        _ => {}
                    }
                }
            }
        } else {
            for child in node.child_nodes().iter() {
                if node_is_valid(child) {
                    render_object = RenderObject::from(<&Node<'_>>::clone(child).clone());
                }
            }
        }

        render_object.clone()
    }
}
