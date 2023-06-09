use crate::arena_tree::ArenaTree;
use crate::dom::NodeRef;

use super::{CssRule, TreeHandle};

/// 6.1.2. The CSSStyleSheet interface
/// https://drafts.csswg.org/cssom/#the-cssstylesheet-interface
#[derive(Debug, Clone, PartialEq)]
pub struct CssStylesheet<'a> {
    rule_arena: &'a ArenaTree<CssRule>,

    location: Option<String>,
    owner_node: Option<NodeRef<'a>>,
    owner_css_rule: Option<TreeHandle>,
    title: Option<String>,
    alternate: bool,
    disabled: bool,
    css_rules: Option<Vec<TreeHandle>>,
    disallow_modification: bool,
    constructor_document: Option<NodeRef<'a>>,
    stylesheet_base_url: Option<String>,
}

impl<'a> CssStylesheet<'a> {
    // https://drafts.csswg.org/cssom/#concept-css-style-sheet-type
    pub fn type_(&self) -> &str {
        "text/css"
    }

    // https://drafts.csswg.org/cssom/#concept-css-style-sheet-location
    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }

    // https://drafts.csswg.org/cssom/#concept-css-style-sheet-parent-css-style-sheet
    pub fn parent_css_style_sheet(&self) {
        todo!()
    }

    // https://drafts.csswg.org/cssom/#concept-css-style-sheet-owner-node
    pub fn owner_node(&self) -> Option<NodeRef<'a>> {
        self.owner_node
    }

    // https://drafts.csswg.org/cssom/#dom-cssstylesheet-ownerrule
    pub fn owner_css_rule(&self) -> Option<TreeHandle> {
        todo!()
    }

    // https://drafts.csswg.org/cssom/#concept-css-style-sheet-media
    pub fn media(&self) {
        todo!()
    }

    // https://drafts.csswg.org/cssom/#concept-css-style-sheet-title
    pub fn title(&self) -> Option<&str> {
        todo!()
    }

    // FIXME: This should return a RuleList.
    // FIXME: SPECLINK
    pub fn css_rules(&self) -> Vec<CssRule> {
        match self.css_rules {
            Some(ref handles) => handles
                .iter()
                .map(|handle| self.rule_arena.get(*handle).value().clone())
                .collect(),
            None => vec![],
        }
    }

    // FIXME: SPECLINK
    pub fn insert_rule(&self, _rule: &str, _index: Option<usize>) -> usize {
        todo!()
    }

    // FIXME: SPECLINK
    pub fn delete_rule(&self, _index: usize) {
        todo!()
    }

    // FIXME: SPECLINK
    pub fn replace(&self, _text: &str) {
        todo!()
    }

    // FIXME: SPECLINK
    pub fn replace_sync(&self, _text: &str) {
        todo!()
    }
}

impl<'a> CssStylesheet<'a> {
    pub fn new(
        rule_arena: &'a ArenaTree<CssRule>,
        location: Option<&str>,
        css_rules: Vec<CssRule>,
    ) -> Self {
        Self {
            rule_arena,
            location: location.map(|s| s.to_string()),
            owner_node: None,
            owner_css_rule: None,
            title: None,
            alternate: false,
            disabled: false,
            css_rules: None,
            disallow_modification: false,
            constructor_document: None,
            stylesheet_base_url: None,
        }
    }
}
