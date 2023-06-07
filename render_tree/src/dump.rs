use crate::{RenderNode, RenderTree};

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

impl RenderTree<'_> {
    pub fn dump(&self, settings: DumpSettings) {
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

        self.tree.dump(
            "  ",
            |node| match node.value() {
                RenderNode::Text(data) => {
                    Some(format!("{gray}#text \"{white}{}{gray}\"{reset}", {
                        match settings.trim_text {
                            true => data.trim().to_string(),
                            false => data.clone(),
                        }
                    }))
                }
                RenderNode::Element(element) if element.is_element() => {
                    Some(format!("{yellow}{}{reset}", element.node_name()))
                }
                _ => None,
            },
            |_| settings.closing_marker.map(|marker| marker.to_string()),
        );
    }
}
