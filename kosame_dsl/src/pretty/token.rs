use std::borrow::Cow;

use crate::pretty::RingBuffer;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TextMode {
    Always,
    NoBreak,
    Break,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BreakMode {
    Consistent,
    Inconsistent,
}

#[derive(Debug)]
pub(super) enum Token<'a> {
    Text(TextToken<'a>),
    Break(BreakToken),
    Begin(BeginToken),
    End,
}

#[derive(Debug)]
pub(super) struct TextToken<'a> {
    string: Cow<'a, str>,
    mode: TextMode,
}

impl<'a> TextToken<'a> {
    pub(super) fn new(string: Cow<'a, str>, mode: TextMode) -> Self {
        Self { string, mode }
    }

    pub(super) fn string(&self) -> &str {
        &self.string
    }

    pub(super) fn mode(&self) -> TextMode {
        self.mode
    }
}

#[derive(Debug)]
pub(super) struct BreakToken {
    len: isize,
    indent: isize,
    force: bool,
}

impl BreakToken {
    pub(super) fn new(len: isize, indent: isize, force: bool) -> Self {
        Self { len, indent, force }
    }

    pub fn push_len(&mut self, len: isize) {
        self.len += len;
    }

    pub(super) fn len(&self) -> isize {
        self.len
    }

    pub(super) fn indent(&self) -> isize {
        self.indent
    }

    pub(super) fn force(&self) -> bool {
        self.force
    }
}

#[derive(Debug)]
pub(super) struct BeginToken {
    mode: BreakMode,
    len: isize,
}

impl BeginToken {
    pub fn push_len(&mut self, len: isize) {
        self.len += len;
    }

    pub(super) fn mode(&self) -> BreakMode {
        self.mode
    }

    pub fn len(&self) -> isize {
        self.len
    }
}

impl BeginToken {
    pub(super) fn new(mode: BreakMode, len: isize) -> Self {
        Self { mode, len }
    }
}

pub(super) struct TokenBuffer<'a> {
    tokens: RingBuffer<Token<'a>>,
    last_break: Option<usize>,
    begin_stack: Vec<usize>,
}

impl<'a> TokenBuffer<'a> {
    pub fn new() -> Self {
        Self {
            tokens: RingBuffer::new(),
            last_break: None,
            begin_stack: Vec::new(),
        }
    }

    pub fn last_break_mut(&mut self) -> Option<&mut BreakToken> {
        match &self.last_break {
            Some(index) => match &mut self.tokens[*index] {
                Token::Break(break_token) => Some(break_token),
                _ => unreachable!(),
            },
            _ => None,
        }
    }

    pub fn current_begin_mut(&mut self) -> Option<&mut BeginToken> {
        match self.begin_stack.last() {
            Some(index) => match &mut self.tokens[*index] {
                Token::Begin(begin_token) => Some(begin_token),
                _ => unreachable!(),
            },
            _ => None,
        }
    }

    pub fn push_len(&mut self, len: isize) {
        if let Some(last_break) = self.last_break_mut() {
            last_break.push_len(len);
        }
        if let Some(current_begin) = self.current_begin_mut() {
            current_begin.push_len(len);
        }
    }

    pub fn push_back(&mut self, token: Token<'a>) {
        match &token {
            Token::Text(_) => {}
            Token::Break(_) => self.last_break = Some(self.tokens.next_index()),
            Token::Begin(_) => self.begin_stack.push(self.tokens.next_index()),
            Token::End => {
                self.begin_stack.pop();
                self.last_break = None;
            }
        }
        self.tokens.push_back(token);
    }

    pub fn pop_front(&mut self) -> Option<Token<'a>> {
        self.tokens.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}
