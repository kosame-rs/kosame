use std::borrow::Cow;

use super::{PrettyPrint, RingBuffer, Span, Trivia};

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
        space: bool,
        len: isize,
    },
    Begin {
        mode: BreakMode,
        len: isize,
    },
    End,
}

impl Token<'_> {
    fn len(&self) -> isize {
        match self {
            Self::Text { string, .. } => string.len().try_into().unwrap(),
            Self::Break { len, .. } | Self::Begin { len, .. } => *len,
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
    indent: isize,
    tokens: RingBuffer<Token<'a>>,
    last_break: Option<usize>,
    begin_stack: Vec<usize>,
    print_frames: Vec<PrintFrame>,
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

    pub fn scan_text(&mut self, string: Cow<'static, str>, mode: TextMode) {
        let token = Token::Text { string, mode };
        self.push_len(token.len());
        self.tokens.push_back(token);
    }

    pub fn scan_break(&mut self, space: bool) {
        self.last_break = Some(self.tokens.len());
        let len = isize::from(space);
        self.tokens.push_back(Token::Break { space, len });
        self.push_len(len);
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
        let begin_len = self.tokens[begin_index].len();

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

    fn print_break(&mut self) {
        self.output.push('\n');
        self.output
            .push_str(&" ".repeat((self.indent * INDENT).try_into().unwrap()));
        self.space = (MARGIN - self.indent * INDENT).max(MIN_SPACE);
    }

    fn print_first(&mut self) {
        let token = self.tokens.pop_front().expect("no tokens to print");

        let group_break = self
            .print_frames
            .last()
            .is_some_and(|frame| frame.group_break);
        let content_break = self
            .print_frames
            .last()
            .is_some_and(|frame| frame.content_break);

        match &token {
            Token::Text { string, mode } => {
                let should_print = matches!(
                    (mode, content_break),
                    (TextMode::Always, _) | (TextMode::Break, true) | (TextMode::NoBreak, false)
                );
                if should_print {
                    self.output.push_str(string);
                    self.space -= isize::try_from(string.len()).unwrap();
                }
            }
            Token::Break { space, len } => {
                if group_break || *len >= self.space {
                    self.print_break();
                } else if *space {
                    self.output.push(' ');
                    self.space -= 1isize;
                }
            }
            Token::Begin { mode, len, .. } => {
                let group_break = *len >= self.space && *mode == BreakMode::Consistent;
                let content_break = *len >= self.space;
                self.print_frames.push(PrintFrame {
                    group_break,
                    content_break,
                });
                self.indent += 1;
            }
            Token::End => {
                self.print_frames.pop();
                self.indent -= 1;
            }
        }
    }

    /// Forces the current print frame to break.
    pub fn scan_force_break(&mut self) {
        self.push_len(MARGIN);
    }

    /// Scans the first trivia element and convert it to tokens.
    fn scan_next_trivia(&mut self) {
        let trivia = &self.trivia[0];
        trivia.pretty_print(self);
        self.trivia = &self.trivia[1..];
        if let Some(next) = self.trivia.first()
            && next.span.immediately_follows(&trivia.span)
        {
            self.scan_next_trivia();
        }
    }

    /// Flushes all trivia that appears before the given token span.
    /// This should be called before structural operations like `scan_begin` to ensure
    /// comments appear in the right place.
    pub fn flush_trivia(&mut self, token_span: Span) {
        while !self.trivia.is_empty() && self.trivia[0].span.comes_before(&token_span) {
            self.scan_next_trivia();
        }
    }

    #[must_use]
    pub fn eof(mut self) -> String {
        while !self.trivia.is_empty() {
            self.scan_next_trivia();
        }

        while !self.tokens.is_empty() {
            self.print_first();
        }

        self.output
    }
}
