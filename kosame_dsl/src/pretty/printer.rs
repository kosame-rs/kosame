use std::borrow::Cow;

use proc_macro2::LineColumn;

use crate::pretty::TriviaKind;

use super::{RingBuffer, Trivia};

pub const MARGIN: isize = 89;
pub const INDENT: isize = 4;
pub const MIN_SPACE: isize = 60;

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

enum Token<'a> {
    Text {
        string: Cow<'a, str>,
        mode: TextMode,
    },
    Break {
        len: isize,
        force: bool,
    },
    Indent {
        indent: isize,
    },
    Begin {
        mode: BreakMode,
        len: isize,
    },
    End,
}

#[derive(Debug)]
struct PrintFrame {
    group_break: bool,
}

pub struct Printer<'a> {
    trivia: &'a [Trivia<'a>],
    output: String,
    space: isize,
    indent: isize,
    tokens: RingBuffer<Token<'a>>,
    last_break: Option<usize>,
    begin_stack: Vec<usize>,
    print_frames: Vec<PrintFrame>,
    cursor: LineColumn,
    pending_break: bool,
}

impl<'a> Printer<'a> {
    #[must_use]
    pub fn new(trivia: &'a [Trivia<'a>], initial_space: isize, initial_indent: isize) -> Self {
        Self {
            trivia,
            output: String::new(),
            space: initial_space.max(MIN_SPACE),
            indent: initial_indent,
            tokens: RingBuffer::new(),
            last_break: None,
            begin_stack: Vec::new(),
            print_frames: Vec::new(),
            cursor: LineColumn { line: 1, column: 0 },
            pending_break: false,
        }
    }

    /// Registers a new token length to be tracked in the previous break and the surrounding
    /// begin/end frame.
    fn push_len(&mut self, token_len: isize) {
        // Track the length that the previous break token has to have available to not break.
        if let Some(break_index) = self.last_break {
            match &mut self.tokens[break_index] {
                Token::Break { len, .. } => *len += token_len,
                _ => unreachable!(),
            }
        }

        // Track the length of the entire begin/end block.
        if let Some(begin_index) = self.begin_stack.last() {
            match &mut self.tokens[*begin_index] {
                Token::Begin { len, .. } => *len += token_len,
                _ => unreachable!(),
            }
        }
    }

    pub fn move_cursor(&mut self, cursor: LineColumn) {
        self.cursor = cursor;
    }

    pub fn scan_text(&mut self, string: Cow<'static, str>, mode: TextMode) {
        self.push_len(string.len().try_into().unwrap());
        for char in string.chars() {
            match char {
                '\n' => {
                    self.cursor.line += 1;
                    self.cursor.column = 0;
                }
                _ => self.cursor.column += 1,
            }
        }
        let token = Token::Text { string, mode };
        self.tokens.push_back(token);
    }

    pub fn scan_break(&mut self, force: bool) {
        self.last_break = Some(self.tokens.len());
        let len = if force { MARGIN } else { 0 };
        self.tokens.push_back(Token::Break { len, force });
        self.push_len(len);
    }

    pub fn scan_indent(&mut self, indent: isize) {
        self.tokens.push_back(Token::Indent { indent });
    }

    pub fn scan_begin(&mut self, mode: BreakMode) {
        self.begin_stack.push(self.tokens.len());
        self.tokens.push_back(Token::Begin { mode, len: 0 });
    }

    /// # Panics
    ///
    /// Panics if there was no matching call to [`scan_begin`] prior to running this function.
    pub fn scan_end(&mut self) {
        let begin_index = self
            .begin_stack
            .pop()
            .expect("printed end without matching begin");
        let Token::Begin { len: begin_len, .. } = self.tokens[begin_index] else {
            unreachable!()
        };

        // Add the length of this begin/end block to its parent.
        if let Some(begin_index) = self.begin_stack.last() {
            match &mut self.tokens[*begin_index] {
                Token::Begin { len, .. } => *len += begin_len,
                _ => unreachable!(),
            }
        }

        self.last_break = None;
        self.tokens.push_back(Token::End);
    }

    pub fn scan_no_break_trivia(&mut self) {
        while let Some(trivia) = self.ready_trivia() {
            match trivia.kind {
                TriviaKind::BlockComment => {
                    self.scan_text(" ".into(), TextMode::Always);
                    self.scan_text(trivia.content.to_string().into(), TextMode::Always);
                }
                TriviaKind::LineComment => {
                    self.scan_text(" ".into(), TextMode::Always);
                    self.scan_text(trivia.content.to_string().into(), TextMode::Always);
                    self.scan_break(true);
                }
                TriviaKind::Whitespace => {}
            }
        }
    }

    pub fn scan_trivia(&mut self) {
        let mut queued_newlines = 0;
        while let Some(trivia) = self.ready_trivia() {
            match trivia.kind {
                TriviaKind::LineComment | TriviaKind::BlockComment => {
                    for _ in 0..queued_newlines {
                        self.scan_break(false);
                    }
                    queued_newlines = 0;
                    self.scan_text(" ".into(), TextMode::Always);
                    self.scan_text(trivia.content.to_string().into(), TextMode::Always);
                }
                TriviaKind::Whitespace => {
                    queued_newlines += trivia.newlines();
                }
            }
        }
    }

    fn ready_trivia(&mut self) -> Option<&'a Trivia<'a>> {
        if let Some(trivia) = self.trivia.first()
            && trivia.span.start() <= self.cursor
        {
            self.trivia = &self.trivia[1..];
            self.cursor = trivia.span.end();
            return Some(trivia);
        }
        None
    }

    fn line_dirty(&self) -> bool {
        if let Some(last) = self.output.chars().last() {
            return last != '\n';
        }
        true
    }

    fn print_string(&mut self, mut string: &str) {
        if self.pending_break {
            self.print_break();
        }
        if !self.line_dirty() {
            string = string.trim_start();
        }
        self.print_indent();
        self.output.push_str(string);
        self.space -= isize::try_from(string.len()).unwrap();
    }

    fn print_break(&mut self) {
        self.output.push('\n');
        self.pending_break = false;
        self.space = MARGIN;
    }

    fn print_indent(&mut self) {
        if !self.line_dirty() {
            self.output
                .push_str(&" ".repeat((self.indent * INDENT).try_into().unwrap()));
            self.space = (self.space - self.indent * INDENT).max(MIN_SPACE);
        }
    }

    fn print_first(&mut self) {
        let token = self.tokens.pop_front().expect("no tokens to print");

        let group_break = self
            .print_frames
            .last()
            .is_some_and(|frame| frame.group_break);

        match &token {
            Token::Text { string, mode } => {
                let should_print = matches!(
                    (mode, group_break),
                    (TextMode::Always, _) | (TextMode::Break, true) | (TextMode::NoBreak, false)
                );
                if should_print {
                    self.print_string(string);
                }
            }
            Token::Break { len, force } => {
                if group_break || *len >= self.space || *force {
                    self.print_break();
                }
            }
            Token::Indent { indent } => {
                self.indent += indent;
            }
            Token::Begin { mode, len, .. } => {
                self.print_indent();
                let group_break = *len >= self.space && *mode == BreakMode::Consistent;
                self.print_frames.push(PrintFrame { group_break });
            }
            Token::End => {
                self.print_frames.pop();
            }
        }
    }

    /// Flushes all trivia that appears before the given token span.
    /// This should be called before structural operations like `scan_begin` to ensure
    /// comments appear in the right place.
    pub fn flush_trivia(&mut self) {
        while let Some(trivia) = self.ready_trivia() {
            match trivia.kind {
                TriviaKind::BlockComment => {
                    self.scan_text(trivia.content.to_string().into(), TextMode::Always);
                    self.scan_break(false);
                    self.scan_text(" ".into(), TextMode::Always);
                }
                TriviaKind::LineComment => {
                    self.scan_text(" ".into(), TextMode::Always);
                    self.scan_text(trivia.content.to_string().into(), TextMode::Always);
                    self.scan_break(true);
                }
                TriviaKind::Whitespace => {}
            }
        }
    }

    #[must_use]
    pub fn eof(mut self) -> String {
        while !self.tokens.is_empty() {
            self.print_first();
        }

        self.output
    }
}
