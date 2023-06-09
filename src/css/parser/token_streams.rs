use std::cell::{Cell, RefCell};

use crate::css::tokenizer::Token;

/// 5.3. Token Streams
///
/// https://drafts.csswg.org/css-syntax-3/#parser-definitions
pub struct TokenStream {
    /// A list of tokens and/or component values.
    ///
    /// https://drafts.csswg.org/css-syntax-3/#token-stream-tokens
    tokens: Vec<Token>,

    /// An index into the tokens, representing the progress of parsing. It starts at 0 initially.
    ///
    /// https://drafts.csswg.org/css-syntax-3/#token-stream-index
    index: Cell<usize>,

    /// A stack of index values, representing points that the parser might return to. It starts empty initially.
    ///
    /// https://drafts.csswg.org/css-syntax-3/#token-stream-marked-indexes
    marked_indexes: RefCell<Vec<usize>>,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> TokenStream {
        TokenStream {
            tokens,
            index: Cell::new(0),
            marked_indexes: RefCell::new(vec![]),
        }
    }

    /// https://drafts.csswg.org/css-syntax-3/#token-stream-next-token
    pub fn next_token(&self) -> &Token {
        // The item of tokens at index.
        self.tokens
            .get(self.index.get())
            // If that index would be out-of-bounds past the end of the list, it’s instead an <eof-token>.
            .unwrap_or(&Token::EndOfFile)
    }

    /// https://drafts.csswg.org/css-syntax-3/#token-stream-empty
    pub fn empty(&self) -> bool {
        // A token stream is empty if the next token is an <eof-token>.
        self.next_token() == &Token::EndOfFile
    }

    /// https://drafts.csswg.org/css-syntax-3/#token-stream-consume-a-token
    pub fn consume_a_token(&self) -> &Token {
        // Let token be the next token.
        let token = self.next_token();
        // Increment index,
        self.index.set(self.index.get() + 1);
        // then return token.
        token
    }

    /// https://drafts.csswg.org/css-syntax-3/#token-stream-discard-a-token
    pub fn discard_a_token(&self) {
        // If the token stream is not empty, increment index.
        if !self.empty() {
            self.index.set(self.index.get() + 1);
        }
    }

    /// https://drafts.csswg.org/css-syntax-3/#token-stream-mark
    pub fn mark(&self) {
        // Append index to marked indexes.
        self.marked_indexes.borrow_mut().push(self.index.get());
    }

    /// https://drafts.csswg.org/css-syntax-3/#token-stream-restore-a-mark
    pub fn restore_a_mark(&self) {
        // Pop from marked indexes, and set index to the popped value.
        let popped = self.marked_indexes.borrow_mut().pop().unwrap();
        self.index.set(popped);
    }

    /// https://drafts.csswg.org/css-syntax-3/#token-stream-discard-a-mark
    pub fn discard_a_mark(&self) {
        // Pop from marked indexes, and do nothing with the popped value.
        self.marked_indexes.borrow_mut().pop();
    }

    /// https://drafts.csswg.org/css-syntax-3/#token-stream-discard-whitespace
    pub fn discard_whitespace(&self) {
        // While the next token is a <whitespace-token>, discard a token.
        while self.next_token() == &Token::Whitespace {
            self.discard_a_token();
        }
    }

    /// https://drafts.csswg.org/css-syntax-3/#token-stream-process
    pub fn process<A, R>(&self, mut action: A) -> R
    where
        A: FnMut(&Token) -> ProcessResult<R>,
    {
        loop {
            match action(self.next_token()) {
                ProcessResult::Return(result) => return result,
                ProcessResult::Continue => {}
            }
        }
    }
}

pub enum ProcessResult<T> {
    Continue,
    Return(T),
}
