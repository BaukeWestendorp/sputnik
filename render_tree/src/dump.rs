use crate::RenderObject;

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
