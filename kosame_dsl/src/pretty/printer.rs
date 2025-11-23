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

enum Token<'a> {
    Text {
        text: Cow<'static, str>,
        span: Option<Span>,
        mode: TextMode,
    },
    Trivia(&'a Trivia<'a>),
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

impl<'a> Token<'a> {
    fn len(&self) -> usize {
        match self {
            Self::Text { text, .. } => text.len(),
            Self::Trivia(trivia) => trivia.content.len(),
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
    tokens: VecDeque<Token<'a>>,
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

    fn token(&self, index: usize) -> &Token<'a> {
        &self.tokens[index]
    }

    fn token_mut(&mut self, index: usize) -> &mut Token<'a> {
        &mut self.tokens[index]
    }

    pub fn scan_text(&mut self, text: impl Text) {
        self.scan_text_with_mode(text, TextMode::Always);
    }

    pub fn scan_text_with_mode(&mut self, text: impl Text, mode: TextMode) {
        let span = text.span();

        // Scan any trivia that appears before this token
        if let Some(token_span) = span {
            self.scan_trivia_before(token_span);
        }

        let text = text.into_cow_str();
        let text_len = text.len();
        self.tokens.push_back(Token::Text { text, span, mode });

        // Track the length that the previous break token has to have available to not break.
        if let Some(break_index) = self.last_break {
            match self.token_mut(break_index) {
                Token::Break { len, .. } => *len += text_len,
                _ => unreachable!(),
            }
        }

        // Track the length of the entire begin/end block.
        if let Some(begin_index) = self.begin_stack.last() {
            match self.token_mut(*begin_index) {
                Token::Begin { len, .. } => *len += text_len,
                _ => unreachable!(),
            }
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
            Token::Text { text, mode, span } => {
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
            Token::Trivia(trivia) => {
                match trivia.kind {
                    TriviaKind::LineComment | TriviaKind::BlockComment => {
                        self.output.push_str(trivia.content);
                        self.space -= trivia.content.len() as isize;
                    }
                    TriviaKind::Whitespace => {
                        // For whitespace, we generally let the pretty printer control spacing
                        // But we should preserve newlines that appear in the original source
                        if trivia.content.contains('\n') {
                            // If there are newlines, we might want to preserve them
                            // For now, let the printer handle breaks
                        }
                    }
                }
            }
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

    /// Scan trivia that appears before the given token span
    fn scan_trivia_before(&mut self, token_span: Span) {
        let token_start = token_span.start();

        while !self.trivia.is_empty() {
            let trivia = &self.trivia[0];

            // Check if this trivia comes before the token
            // We need to compare proc_macro2 line/column with our trivia span
            let trivia_end_line = trivia.span.end_line;
            let trivia_end_col = trivia.span.end_col;

            let token_line = token_start.line;
            let token_col = token_start.column;

            // Trivia comes before token if:
            // - It ends on an earlier line, OR
            // - It ends on the same line but before the token column
            let trivia_before_token = trivia_end_line < token_line
                || (trivia_end_line == token_line && trivia_end_col <= token_col);

            if !trivia_before_token {
                break;
            }

            // Scan this trivia as a token
            let trivia_len = trivia.content.len();
            self.tokens.push_back(Token::Trivia(trivia));

            // Track the length for break calculations
            if let Some(break_index) = self.last_break {
                match self.token_mut(break_index) {
                    Token::Break { len, .. } => *len += trivia_len,
                    _ => unreachable!(),
                }
            }

            // Track the length of the entire begin/end block
            if let Some(begin_index) = self.begin_stack.last() {
                match self.token_mut(*begin_index) {
                    Token::Begin { len, .. } => *len += trivia_len,
                    _ => unreachable!(),
                }
            }

            // Move to next trivia
            self.trivia = &self.trivia[1..];
        }
    }

    /// Scan all remaining trivia at the end
    fn scan_remaining_trivia(&mut self) {
        while !self.trivia.is_empty() {
            let trivia = &self.trivia[0];
            let trivia_len = trivia.content.len();
            self.tokens.push_back(Token::Trivia(trivia));

            // Track the length for break calculations
            if let Some(break_index) = self.last_break {
                match self.token_mut(break_index) {
                    Token::Break { len, .. } => *len += trivia_len,
                    _ => unreachable!(),
                }
            }

            // Track the length of the entire begin/end block
            if let Some(begin_index) = self.begin_stack.last() {
                match self.token_mut(*begin_index) {
                    Token::Begin { len, .. } => *len += trivia_len,
                    _ => unreachable!(),
                }
            }

            // Move to next trivia
            self.trivia = &self.trivia[1..];
        }
    }

    pub fn eof(mut self) -> String {
        // Scan any remaining trivia
        self.scan_remaining_trivia();

        // Print all tokens (including scanned trivia)
        while !self.tokens.is_empty() {
            self.print_first();
        }

        self.output
    }
}
