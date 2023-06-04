use std::cell::{Cell, RefCell};

use dom::{Node, NodeType};
use tokenizer::{Token, Tokenizer};
use tree_construction::insertion_modes;
use tree_construction::list_of_active_formatting_elements::ListOfActiveFormattingElements;
use tree_construction::stack_of_open_elements::StackOfOpenElements;
use typed_arena::Arena;
use types::{InsertionMode, NodeLink, NodeRef};

pub mod dom;
pub mod namespace;
pub(crate) mod tree_construction;
pub mod types;

const fn is_parser_whitespace(string: char) -> bool {
    if let '\t' | '\u{000a}' | '\u{000c}' | '\u{000d}' | '\u{0020}' = string {
        return true;
    }
    false
}

macro_rules! log_current_process {
    ($insertion_mode:expr, $token:expr) => {
        if std::env::var("PARSER_LOGGING").is_ok() {
            eprintln!(
                "\x1b[32m[Parser::InsertionMode::{:?}] {:?}\x1b[0m",
                $insertion_mode, $token
            );
        }
    };
}

pub struct Parser<'a> {
    arena: Arena<Node<'a>>,
    tokenizer: RefCell<Tokenizer>,
    document: Node<'a>,
    insertion_mode: Cell<InsertionMode>,
    open_elements: StackOfOpenElements<'a>,
    active_formatting_elements: ListOfActiveFormattingElements<'a>,
    head_element: NodeLink<'a>,
    form_element: NodeLink<'a>,
    scripting: bool,
    frameset_ok: Cell<bool>,
}

impl<'a> Parser<'a> {
    pub fn new(arena: Arena<Node<'a>>, input: &str) -> Self {
        Self {
            arena,
            tokenizer: RefCell::new(Tokenizer::new(input)),
            document: Node::new(None, NodeType::Document),
            insertion_mode: Cell::new(InsertionMode::Initial),
            open_elements: StackOfOpenElements::new(),
            active_formatting_elements: ListOfActiveFormattingElements::new(),
            head_element: Cell::new(None),
            form_element: Cell::new(None),
            scripting: false,
            frameset_ok: Cell::new(false),
        }
    }

    pub(crate) fn allocate_node(&'a self, node: Node<'a>) -> NodeRef<'a> {
        self.arena.alloc(node)
    }

    pub(crate) fn process_token_using_the_rules_for(
        &'a self,
        insertion_mode: InsertionMode,
        token: &Token,
    ) {
        log_current_process!(insertion_mode, token);

        match insertion_mode {
            InsertionMode::Initial => self.handle_initial(token),
            InsertionMode::BeforeHtml => self.handle_before_html(token),
            InsertionMode::BeforeHead => self.handle_before_head(token),
            InsertionMode::InHead => self.handle_in_head(token),
            InsertionMode::AfterHead => self.handle_after_head(token),
            InsertionMode::InBody => self.handle_in_body(token),
            _ => todo!("{:?}", insertion_mode),
        }
    }

    fn process_token(&'a self, token: &Token) {
        self.process_token_using_the_rules_for(self.insertion_mode.get(), token);
    }

    fn process_token_in_foreign_context(&self, token: &Token) {
        log_current_process!(self.insertion_mode, token);

        match token {
            Token::Character { data } if data == &'\u{0000}' => todo!(),
            Token::Character { data } if is_parser_whitespace(*data) => todo!(),
            Token::Character { .. } => todo!(),
            Token::Comment { .. } => todo!(),
            Token::Doctype { .. } => todo!(),
            Token::StartTag {
                name, attributes, ..
            } if name == "b"
                || name == "big"
                || name == "blockquote"
                || name == "body"
                || name == "br"
                || name == "center"
                || name == "code"
                || name == "dd"
                || name == "div"
                || name == "dl"
                || name == "dt"
                || name == "em"
                || name == "embed"
                || name == "h1"
                || name == "h2"
                || name == "h3"
                || name == "h4"
                || name == "h5"
                || name == "h6"
                || name == "head"
                || name == "hr"
                || name == "i"
                || name == "img"
                || name == "li"
                || name == "listing"
                || name == "menu"
                || name == "meta"
                || name == "nobr"
                || name == "ol"
                || name == "p"
                || name == "pre"
                || name == "ruby"
                || name == "s"
                || name == "small"
                || name == "span"
                || name == "strong"
                || name == "strike"
                || name == "sub"
                || name == "sup"
                || name == "table"
                || name == "tt"
                || name == "u"
                || name == "ul"
                || name == "var"
                || (name == "font"
                    && attributes.iter().any(|attr| {
                        attr.name == "color" || attr.name == "face" || attr.name == "size"
                    })) =>
            {
                todo!()
            }
            Token::EndTag { name, .. } if name == "br" || name == "P" => todo!(),
            Token::StartTag { .. } => todo!(),
            // FIXME: An end tag whose tag name is "script", if the current node is an SVG script element
            _ => {
                // 1. Initialize node to be the current node (the bottommost node of the stack).
                let node = self.open_elements.current_node().unwrap();

                // 2. If node's tag name, converted to ASCII lowercase, is not the same as the tag name of the token, then this is a parse error.
                if let Some(tag_name) = node.element_tag_name() {
                    if !tag_name.eq_ignore_ascii_case(&token.tag_name().unwrap()) {
                        todo!();
                    }
                }

                // FIXME: 3. Loop: If node is the topmost element in the stack of open elements, then return. (fragment case)
                // FIXME: 4. If node's tag name, converted to ASCII lowercase, is the same as the tag name of the token, pop elements from the stack of open elements until node has been popped from the stack, and then return.
                // FIXME: 5. Set node to the previous entry in the stack of open elements.
                // FIXME: 6. If node is not an element in the HTML namespace, return to the step labeled loop.
                // FIXME: 7. Otherwise, process the token according to the rules given in the section corresponding to the current insertion mode in HTML content.
            }
        }
    }

    fn token_is_not_in_foreign_context(&self, token: &Token) -> bool {
        self.open_elements.is_empty() ||
        // FIXME: If the adjusted current node is an element in the HTML namespace
        // FIXME: If the adjusted current node is a MathML text integration point and the token is a start tag whose tag name is neither "mglyph" nor "malignmark"
        // FIXME: If the adjusted current node is a MathML text integration point and the token is a character token
        // FIXME: If the adjusted current node is a MathML annotation-xml element and the token is a start tag whose tag name is "svg"
        // FIXME: If the adjusted current node is an HTML integration point and the token is a start tag
        // FIXME: If the adjusted current node is an HTML integration point and the token is a character token
        // If the token is an end-of-file token
        matches!(token, Token::EndOfFile)
    }

    pub fn parse(&'a self) -> Node<'a> {
        while let Some(token) = self.tokenizer.borrow_mut().next_token() {
            if self.token_is_not_in_foreign_context(token) {
                self.process_token(&token)
            } else {
                self.process_token_in_foreign_context(&token)
            };
        }

        self.document.clone()
    }
}

// 13.2.7 The End
// https://html.spec.whatwg.org/#the-end
impl<'a> Parser<'a> {
    pub(crate) fn stop_parsing(&'a self) {
        // FIXME: 1. If the active speculative HTML parser is not null, then stop the speculative HTML parser and return.
        // FIXME: 2. Set the insertion point to undefined.
        // FIXME: 3. Update the current document readiness to "interactive".
        // 4. Pop all the nodes off the stack of open elements.
        self.open_elements.clear();
        // FIXME: 5. While the list of scripts that will execute when the document has finished parsing is not empty:
        // FIXME: 6. Queue a global task on the DOM manipulation task source given the Document's relevant global object to run the following substeps:
        // FIXME: 7. Spin the event loop until the set of scripts that will execute as soon as possible and the list of scripts that will execute in order as soon as possible are empty.
        // FIXME: 8. Spin the event loop until there is nothing that delays the load event in the Document.
        // FIXME: 9. Queue a global task on the DOM manipulation task source given the Document's relevant global object to run the following steps:
        // FIXME: 10. If the Document's print when loaded flag is set, then run the printing steps.
        // FIXME: 11. The Document is now ready for post-load tasks.
    }
}
