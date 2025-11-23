use std::{borrow::Cow, collections::VecDeque};

use proc_macro2::Span;

use crate::pretty::{Trivia, TriviaKind};

use super::Text;

pub const MARGIN: usize = 89;
pub const INDENT: usize = 4;
pub const MIN_SPACE: usize = 60;

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

enum Token {
    Text {
        text: Cow<'static, str>,
        mode: TextMode,
    },
    Space,
    Break {
        text: Cow<'static, str>,
        len: usize,
    },
    Begin {
        mode: BreakMode,
        len: usize,
    },
    End,
}

impl Token {
    fn len(&self) -> usize {
        match self {
            Self::Text { text, .. } => text.len(),
            Self::Space => 1,
            Self::Break { len, .. } => *len,
            Self::Begin { len, .. } => *len,
            Self::End => 0,
        }
    }
}

#[derive(Debug)]
struct PrintFrame {
    group_break: bool,
    content_break: bool,
}

pub struct Printer<'a> {
    trivia: &'a [Trivia<'a>],
    output: String,
    space: isize,
    indent: usize,
    tokens: VecDeque<Token>,
    last_break: Option<usize>,
    begin_stack: Vec<usize>,
    print_frames: Vec<PrintFrame>,
}

impl<'a> Printer<'a> {
    pub fn new(trivia: &'a [Trivia<'a>], initial_space: usize, initial_indent: usize) -> Self {
        Self {
            trivia,
            output: String::new(),
            space: initial_space.max(MIN_SPACE) as isize,
            indent: initial_indent,
            tokens: VecDeque::new(),
            last_break: None,
            begin_stack: Vec::new(),
            print_frames: Vec::new(),
        }
    }

    fn token(&self, index: usize) -> &Token {
        &self.tokens[index]
    }

    fn token_mut(&mut self, index: usize) -> &mut Token {
        &mut self.tokens[index]
    }

    /// Registers a new token length to be tracked in the previous break and the surrounding
    /// begin/end frame.
    fn push_len(&mut self, token_len: usize) {
        // Track the length that the previous break token has to have available to not break.
        if let Some(break_index) = self.last_break {
            match self.token_mut(break_index) {
                Token::Break { len, .. } => *len += token_len,
                _ => unreachable!(),
            }
        }

        // Track the length of the entire begin/end block.
        if let Some(begin_index) = self.begin_stack.last() {
            match self.token_mut(*begin_index) {
                Token::Begin { len, .. } => *len += token_len,
                _ => unreachable!(),
            }
        }
    }

    pub fn scan_text(&mut self, text: impl Text) {
        self.scan_text_with_mode(text, TextMode::Always);
    }

    pub fn scan_text_with_mode(&mut self, text: impl Text, mode: TextMode) {
        let span = text.span();

        // Flush any trivia that appears before this token
        if let Some(token_span) = span {
            self.flush_trivia(token_span);
        }

        let token = Token::Text {
            text: text.into_cow_str(),
            mode,
        };
        self.push_len(token.len());
        self.tokens.push_back(token);
    }

    pub fn scan_space(&mut self) {
        if !matches!(self.tokens.iter().last(), Some(Token::Space)) {
            self.push_len(1);
            self.tokens.push_back(Token::Space);
        }
    }

    pub fn scan_break(&mut self, text: impl Into<Cow<'static, str>>) {
        let text = text.into();
        let len = text.len();
        self.last_break = Some(self.tokens.len());
        self.tokens.push_back(Token::Break { text, len });
    }

    pub fn scan_begin(&mut self, mode: BreakMode) {
        self.begin_stack.push(self.tokens.len());
        self.tokens.push_back(Token::Begin { mode, len: 0 });
    }

    pub fn scan_end(&mut self) {
        let begin_index = self
            .begin_stack
            .pop()
            .expect("printed end without matching begin");
        let begin_len = self.token(begin_index).len();

        // Add the length of this begin/end block to its parent.
        if let Some(begin_index) = self.begin_stack.last() {
            match self.token_mut(*begin_index) {
                Token::Begin { len, .. } => *len += begin_len,
                _ => unreachable!(),
            }
        }

        self.last_break = None;
        self.tokens.push_back(Token::End);
    }

    fn print_break(&mut self) {
        self.output.push('\n');
        self.output.push_str(&" ".repeat(self.indent * INDENT));
        self.space = (MARGIN - self.indent * INDENT).max(MIN_SPACE) as isize;
    }

    fn print_first(&mut self) {
        let token = self.tokens.pop_front().expect("no tokens to print");

        let content_break = self
            .print_frames
            .last()
            .map(|frame| frame.content_break)
            .unwrap_or(false);

        match &token {
            Token::Text { text, mode, .. } => {
                let should_print = matches!(
                    (mode, content_break),
                    (TextMode::Always, _) | (TextMode::Break, true) | (TextMode::NoBreak, false)
                );
                if should_print {
                    self.output.push_str(text);
                    self.space -= text.len() as isize;
                    println!("{}", text);
                }
            }
            Token::Space => {}
            Token::Break { text, len } => {
                if content_break || *len as isize >= self.space {
                    self.print_break();
                } else {
                    self.output.push_str(text);
                    self.space -= text.len() as isize;
                }
            }
            Token::Begin { mode, len, .. } => {
                let group_break = *len as isize >= self.space && *mode == BreakMode::Consistent;
                self.print_frames.push(PrintFrame {
                    group_break,
                    content_break: group_break,
                });
                self.indent += 1;
                println!("group len {} -> {group_break} >= {}", len, self.space);
                if group_break {
                    self.print_break();
                }
            }
            Token::End => {
                let print_frame = self
                    .print_frames
                    .pop()
                    .expect("emitted end token without begin");
                self.indent -= 1;

                if print_frame.group_break {
                    self.print_break();
                }
            }
        };
    }

    /// Forces the current print frame to break.
    pub fn scan_force_break(&mut self) {
        if let Some(break_index) = self.last_break {
            match self.token_mut(break_index) {
                Token::Break { len, .. } => *len = MARGIN,
                _ => unreachable!(),
            }
        }

        if let Some(begin_index) = self.begin_stack.last() {
            match self.token_mut(*begin_index) {
                Token::Begin { len, .. } => *len = MARGIN,
                _ => unreachable!(),
            }
        }
    }

    /// Scans the first trivia element and convert it to tokens.
    fn scan_first_trivia(&mut self) {
        if self.trivia.is_empty() {
            return;
        }

        let trivia = &self.trivia[0];
        match trivia.kind {
            TriviaKind::LineComment => {
                self.scan_force_break();
                self.scan_text(trivia.content.to_string());
                self.scan_break("");
            }
            TriviaKind::BlockComment => {
                self.scan_text(trivia.content.to_string());
                self.scan_break(" ");
            }
            TriviaKind::Whitespace => {
                if trivia.content.chars().filter(|item| *item == '\n').count() >= 2 {
                    self.scan_break("");
                }
            }
        }

        self.trivia = &self.trivia[1..];
    }

    /// Flushes all trivia that appears before the given token span.
    /// This should be called before structural operations like `scan_begin` to ensure
    /// comments appear in the right place.
    pub fn flush_trivia(&mut self, token_span: Span) {
        while !self.trivia.is_empty() && self.trivia[0].comes_before(token_span) {
            self.scan_first_trivia();
        }
    }

    pub fn eof(mut self) -> String {
        while !self.trivia.is_empty() {
            self.scan_first_trivia();
        }

        while !self.tokens.is_empty() {
            self.print_first();
        }

        self.output
    }
}
