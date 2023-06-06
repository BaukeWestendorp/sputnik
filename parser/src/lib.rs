use std::cell::{Cell, RefCell};

use dom::element::Element;
use dom::node::{Node, NodeLink, NodeRef, NodeType};
use html::namespace::Namespace;
use tokenizer::{Token, Tokenizer};
use tree_construction::list_of_active_formatting_elements::ListOfActiveFormattingElements;
use tree_construction::stack_of_open_elements::StackOfOpenElements;
use typed_arena::Arena;

pub(crate) mod tree_construction;

const fn is_parser_whitespace(string: char) -> bool {
    if let '\t' | '\u{000a}' | '\u{000c}' | '\u{000d}' | '\u{0020}' = string {
        return true;
    }
    false
}

macro_rules! log_current_process {
    ($insertion_mode:expr, $token:expr) => {
        if std::env::var("PARSER_LOGGING").is_ok() {
            eprintln!("\x1b[32m[{}] {:?}\x1b[0m", $insertion_mode, $token);
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
enum GenericParsingAlgorithm {
    RawText,
    RcData,
}

pub struct Parser<'a> {
    arena: Arena<Node<'a>>,
    tokenizer: RefCell<Tokenizer>,
    new_tokenizer_state: Cell<Option<tokenizer::State>>,
    document: Node<'a>,
    insertion_mode: Cell<InsertionMode>,
    original_insertion_mode: Cell<Option<InsertionMode>>,
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
            new_tokenizer_state: Cell::new(None),
            document: Node::new(None, NodeType::Document),
            insertion_mode: Cell::new(InsertionMode::Initial),
            original_insertion_mode: Cell::new(None),
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
        log_current_process!(format!("{:?}", insertion_mode), token);

        match insertion_mode {
            InsertionMode::Initial => self.handle_initial(token),
            InsertionMode::BeforeHtml => self.handle_before_html(token),
            InsertionMode::BeforeHead => self.handle_before_head(token),
            InsertionMode::InHead => self.handle_in_head(token),
            InsertionMode::AfterHead => self.handle_after_head(token),
            InsertionMode::InBody => self.handle_in_body(token),
            InsertionMode::AfterBody => self.handle_after_body(token),
            InsertionMode::AfterAfterBody => self.handle_after_after_body(token),
            InsertionMode::Text => self.handle_text(token),
            _ => todo!("{:?}", insertion_mode),
        }
    }

    pub(crate) fn current_node(&'a self) -> NodeRef<'a> {
        self.open_elements.current_node()
    }

    fn process_token(&'a self, token: &Token) {
        self.process_token_using_the_rules_for(self.insertion_mode.get(), token);
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#generic-rcdata-element-parsing-algorithm
    fn follow_generic_parsing_algorithm(
        &'a self,
        algorithm: GenericParsingAlgorithm,
        token: &Token,
    ) {
        // 1. Insert an HTML element for the token.
        self.insert_html_element_for_token(token);

        // 2. If the algorithm that was invoked is the generic raw text element parsing algorithm, switch the tokenizer to the RAWTEXT state; otherwise the algorithm invoked was the generic RCDATA element parsing algorithm, switch the tokenizer to the RCDATA state.
        match algorithm {
            GenericParsingAlgorithm::RawText => self
                .new_tokenizer_state
                .set(Some(tokenizer::State::RawText)),
            GenericParsingAlgorithm::RcData => {
                self.new_tokenizer_state.set(Some(tokenizer::State::RcData));
            }
        }

        // 3. Let the original insertion mode be the current insertion mode.
        self.original_insertion_mode
            .set(Some(self.insertion_mode.get()));

        // 4. Then, switch the insertion mode to "text".
        self.switch_insertion_mode_to(InsertionMode::Text);
    }

    // https://html.spec.whatwg.org/#parsing-main-inforeign
    fn process_using_the_rules_for_foreign_content(&'a self, token: &Token) {
        log_current_process!("Foreign Content", token);

        macro_rules! pop_invalid_elements {
            ($name:expr) => {
                // Parse error.
                log_parser_error!(format!("Invalid start tag '{}' in foreign context", $name));

                // While the current node is not FIXME(a MathML text integration point, an HTML integration point), or an element in the HTML namespace, pop elements from the stack of open elements.
                while !self.current_node().is_element_with_namespace(Namespace::Html) {
                    self.open_elements.pop();
                }

                // Reprocess the token according to the rules given in the section corresponding to the current insertion mode in HTML content.
                self.process_token(token);
            };
        }

        match token {
            Token::Character { data } if data == &'\u{0000}' => {
                // Parse error. Insert a U+FFFD REPLACEMENT CHARACTER character.
                log_parser_error!();
                self.insert_character('\u{fffd}');
            }
            Token::Character { data } if is_parser_whitespace(*data) => {
                // Insert the token's character.
                self.insert_character(*data);
            }
            Token::Character { data } => {
                // Insert the token's character.
                self.insert_character(*data);
                // Set the frameset-ok flag to "not ok"
                self.frameset_ok.set(false);
            }
            Token::Comment { data } => {
                // Insert a comment.
                self.insert_comment(data)
            }
            Token::Doctype { .. } => {
                // Parse error. Ignore the token.
                log_parser_error!();
            }
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
                pop_invalid_elements!(name);
            }
            Token::EndTag { name, .. } if name == "br" || name == "P" => {
                pop_invalid_elements!(name);
            }
            Token::StartTag {
                self_closing,
                name,
                self_closing_acknowledged,
                ..
            } => {
                // FIXME: If the adjusted current node is an element in the MathML namespace, adjust MathML attributes for the token. (This fixes the case of MathML attributes that are not all lowercase.)
                // FIXME: If the adjusted current node is an element in the SVG namespace, and the token's tag name is one of the ones in the first column of the following table, change the tag name to the name given in the corresponding cell in the second column. (This fixes the case of SVG elements that are not all lowercase.)
                // FIXME: If the adjusted current node is an element in the SVG namespace, adjust SVG attributes for the token. (This fixes the case of SVG attributes that are not all lowercase.)
                // FIXME: Adjust foreign attributes for the token. (This fixes the use of namespaced attributes, in particular XLink in SVG.)

                // Insert a foreign element for the token, FIXME(in the same namespace as the adjusted current node).
                self.insert_foreign_element_for_token(token, Namespace::Html);

                // If the token has its self-closing flag set, then run the appropriate steps from the following list:
                if *self_closing {
                    // -> If the token's tag name is "script", FIXME(and the new current node is in the SVG namespace)
                    if name == "script" {
                        // Acknowledge the token's self-closing flag, FIXME(and then act as described in the steps for a "script" end tag below.)
                        self_closing_acknowledged.set(true);
                    }
                    // -> Otherwise
                    else {
                        // Pop the current node off the stack of open elements and acknowledge the token's self-closing flag.
                        self.open_elements.pop();
                        self_closing_acknowledged.set(true);
                    }
                }
            }
            Token::EndTag { name, .. } if name == "script" => {
                // FIXME: if the current node is an SVG script element
                todo!()
            }
            _ => {
                // 1. Initialize node to be the current node (the bottommost node of the stack).
                let mut node = self.current_node();

                // 2. If node's tag name, converted to ASCII lowercase, is not the same as the tag name of the token, then this is a parse error.
                if let Some(tag_name) = node.element_tag_name() {
                    let token_tag_name = token.tag_name().unwrap();
                    if tag_name.to_ascii_lowercase() != token_tag_name {
                        log_parser_error!(format!(
                            "current node tag name '{}' is not the same as the token tag name '{}",
                            tag_name.to_ascii_lowercase(),
                            token_tag_name.to_ascii_lowercase()
                        ));
                    }
                }

                // 3. Loop: If node is the topmost element in the stack of open elements, then return. (fragment case)
                for (i, _) in self
                    .open_elements
                    .elements
                    .clone()
                    .borrow()
                    .iter()
                    .enumerate()
                {
                    if self.open_elements.first().unwrap() == node {
                        return;
                    }
                    // 4. If node's tag name, converted to ASCII lowercase, is the same as the tag name of the token, pop elements from the stack of open elements until node has been popped from the stack, and then return.
                    if node.element_tag_name().unwrap().to_ascii_lowercase()
                        == token.tag_name().unwrap()
                    {
                        self.open_elements
                            .pop_elements_until_element_has_been_popped(node);
                        return;
                    }
                    // 5. Set node to the previous entry in the stack of open elements.
                    node = self.open_elements.elements.borrow()[i - 1];
                    // 6. If node is not an element in the HTML namespace, return to the step labeled loop.
                    if !node.is_element_with_namespace(Namespace::Html) {
                        continue;
                    }

                    // 7. Otherwise, process the token according to the rules given in the section   corresponding to the current insertion mode in HTML content.
                    self.process_token(token);
                    return;
                }
            }
        }
    }

    fn token_is_not_in_foreign_context(&self, token: &Token) -> bool {
        // If the stack of open elements is empty
        self.open_elements.is_empty() ||
        // FIXME: If the adjusted current node is an element in the HTML namespace
        self.open_elements.adjusted_current_node().is_element_with_namespace(Namespace::Html) ||
        // FIXME: If the adjusted current node is a MathML text integration point and the token is a start tag whose tag name is neither "mglyph" nor "malignmark"
        // FIXME: If the adjusted current node is a MathML text integration point and the token is a character token
        // FIXME: If the adjusted current node is a MathML annotation-xml element and the token is a start tag whose tag name is "svg"
        // FIXME: If the adjusted current node is an HTML integration point and the token is a start tag
        // FIXME: If the adjusted current node is an HTML integration point and the token is a character token
        // If the token is an end-of-file token
        matches!(token, Token::EndOfFile)
    }

    pub fn parse(&'a self) -> Node<'a> {
        let mut tokenizer = self.tokenizer.borrow_mut();
        while let Some(token) = tokenizer.next_token() {
            if self.token_is_not_in_foreign_context(token) {
                self.process_token(token)
            } else {
                self.process_using_the_rules_for_foreign_content(token)
            };

            if let Some(new_tokenizer_state) = self.new_tokenizer_state.get() {
                tokenizer.switch_to(new_tokenizer_state);
                self.new_tokenizer_state.set(None);
            }
        }

        self.document.clone()
    }
}

// DOM Implementations
impl<'a> Parser<'a> {
    // https://dom.spec.whatwg.org/#concept-create-element
    pub(crate) fn create_element(
        &'a self,
        document: NodeRef<'a>,
        local_name: &String,
        namespace: Namespace,
        _prefix: Option<&String>,
        _is: Option<&String>,
        _synchronous_custom_elements: bool,
    ) -> NodeRef<'a> {
        // FIXME: This does not implement any spec functionality yet!
        self.allocate_node(Node::new(
            Some(document),
            NodeType::Element(Element {
                tag_name: local_name.to_owned(),
                namespace: Some(namespace),
                attributes: RefCell::new(vec![]),
            }),
        ))
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
