use crate::layout_box::BoxType;
use crate::tree::LayoutTree;

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

impl LayoutTree<'_> {
    pub fn dump(&self, settings: DumpSettings) {
        self.tree.dump(
            "  ",
            |node| {
                let box_type = match node.value().box_type() {
                    BoxType::Viewport => "Viewport",
                    BoxType::BlockContainer => "BlockContainer",
                };

                Some(format!(
                    "{}({}) @ ({}, {}) {}x{}",
                    box_type,
                    node.value().node().node_name(),
                    node.value().content_width(),
                    node.value().content_height(),
                    node.value().absolute_x(),
                    node.value().absolute_y(),
                ))
            },
            |_| settings.closing_marker.map(|marker| marker.to_string()),
        );
    }
}
