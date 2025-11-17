use std::{borrow::Cow, collections::VecDeque};

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

#[derive(Default)]
pub struct Printer {
    output: String,
    space: isize,
    indent: usize,
    tokens: VecDeque<Token>,
    last_break: Option<usize>,
    begin_stack: Vec<usize>,
    print_frames: Vec<PrintFrame>,
}

impl Printer {
    pub fn new(initial_space: usize, initial_indent: usize) -> Self {
        Self {
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

    pub fn scan_text(&mut self, text: impl Text) {
        self.scan_text_with_mode(text, TextMode::Always);
    }

    pub fn scan_text_with_mode(&mut self, text: impl Text, mode: TextMode) {
        let text = text.into_cow_str();
        let text_len = text.len();
        self.tokens.push_back(Token::Text { text, mode });

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
            Token::Text { text, mode } => {
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

    pub fn eof(mut self) -> String {
        while !self.tokens.is_empty() {
            self.print_first();
        }

        self.output
    }
}
