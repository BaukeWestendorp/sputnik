use dom::node::{DumpSettings, Node, NodeType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderObject<'a> {
    Text(String),
    Element {
        element: Node<'a>,
        children: Vec<RenderObject<'a>>,
        // FIXME: Style
    },
}

impl RenderObject<'_> {
    pub fn dump(&self, settings: DumpSettings) {
        self.internal_dump("", settings);
    }

    fn internal_dump(&self, indentation: &str, settings: DumpSettings) {
        macro_rules! color {
            ($color:literal) => {
                if settings.color {
                    $color
                } else {
                    ""
                }
            };
        }

        let yellow = color!("\x1b[33m");
        let white = color!("\x1b[37m");
        let reset = color!("\x1b[0m");
        let gray = color!("\x1b[90m");

        let opening = match self {
            RenderObject::Text(data) => {
                format!("{gray}#text \"{white}{}{gray}\"{reset}", {
                    match settings.trim_text {
                        true => data.trim().to_string(),
                        false => data.clone(),
                    }
                })
            }
            RenderObject::Element { element, .. } => {
                format!("{yellow}{}{reset}", element.node_name())
            }
        };

        println!("{indentation}{}", opening);
        if let RenderObject::Element { children, .. } = self {
            for child in children.iter() {
                let mut indentation = indentation.to_string();
                indentation.push_str("  ");
                child.internal_dump(&indentation, settings);
            }
        }

        if let Some(closing_marker) = settings.closing_marker {
            println!("{indentation}{closing_marker}");
        }
    }
}

impl<'a> From<Node<'a>> for RenderObject<'a> {
    fn from(node: Node<'a>) -> Self {
        if let NodeType::Text { data } = node.node_type {
            return RenderObject::Text(data.borrow().clone());
        }

        let mut render_object = RenderObject::Element {
            element: node.clone(),
            children: vec![],
        };

        let node_is_valid = |node: &Node<'a>| match &node.node_type {
            NodeType::Element(element) => {
                element.tag_name != "html"
                    && element.tag_name != "head"
                    && element.tag_name != "script"
                    && element.tag_name != "meta"
                    && element.tag_name != "link"
                    && element.tag_name != "style"
            }
            NodeType::Text { .. } => true,
            _ => false,
        };

        let next_valid_child = |node: &Node<'a>| {
            let mut valid_child = None;
            for child in node.child_nodes().iter() {
                if node_is_valid(&child) {
                    valid_child = Some(<&Node<'_>>::clone(child).clone());
                    break;
                }
            }
            valid_child
        };

        if node_is_valid(&node) {
            match node.node_type {
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
            while let Some(next_sibling) = node.next_sibling() {
                if let Some(next_valid_child) = next_valid_child(next_sibling) {
                    render_object = RenderObject::from(next_valid_child.clone().clone());
                    break;
                }
                if node_is_valid(next_sibling) {
                    render_object = RenderObject::from(next_sibling.clone());
                    break;
                }
            }
            for child in node.child_nodes().iter() {
                if let Some(next_valid_child) = next_valid_child(child) {
                    render_object = RenderObject::from(next_valid_child.clone().clone());
                    break;
                }
                if node_is_valid(child) {
                    render_object = RenderObject::from(<&Node<'_>>::clone(child).clone());
                    break;
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
