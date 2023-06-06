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

        let opening_marker = match self {
            RenderObject::Text(data) => Some(format!("{gray}#text \"{white}{}{gray}\"{reset}", {
                match settings.trim_text {
                    true => data.trim().to_string(),
                    false => data.clone(),
                }
            })),
            RenderObject::Element { element, .. } if element.is_element() => {
                Some(format!("{yellow}{}{reset}", element.node_name()))
            }
            _ => None,
        };

        if let Some(opening_marker) = opening_marker {
            println!("{indentation}{}", opening_marker);
        }
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
                } else {
                    if let Some(next_valid_descendant) = next_valid_render_tree_descendant(child) {
                        todo!()
                    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DumpSettings {
    pub closing_marker: Option<&'static str>,
    pub color: bool,
    pub trim_text: bool,
    pub indentation: &'static str,
}

impl Default for DumpSettings {
    fn default() -> Self {
        Self {
            closing_marker: None,
            color: true,
            trim_text: true,
            indentation: "  ",
        }
    }
}
