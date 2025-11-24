use crate::pretty::{DelimText, TextMode};

use super::{PrettyPrint, RingBuffer, Span, Text, Trivia};

pub const MARGIN: usize = 89;
pub const INDENT: usize = 4;
pub const MIN_SPACE: usize = 60;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BreakMode {
    Consistent,
    Inconsistent,
}

enum Token {
    Text(Text),
    Break { space: bool, len: usize },
    Begin { mode: BreakMode, len: usize },
    End,
}

impl Token {
    fn len(&self) -> usize {
        match self {
            Self::Text(text) => text.len(),
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
    tokens: RingBuffer<Token>,
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
            tokens: RingBuffer::new(),
            last_break: None,
            begin_stack: Vec::new(),
            print_frames: Vec::new(),
        }
    }

    /// Registers a new token length to be tracked in the previous break and the surrounding
    /// begin/end frame.
    fn push_len(&mut self, token_len: usize) {
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

    pub fn scan_text(&mut self, text: impl Into<Text>) {
        let text = text.into();
        let span = text.span();

        // Flush any trivia that appears before this token
        if let Some(token_span) = span {
            self.flush_trivia(token_span);
        }

        let token = Token::Text(text);
        self.push_len(token.len());
        self.tokens.push_back(token);

        // if let Some(token_span) = &span
        //     && let Some(trivia) = self.trivia.first()
        //     && trivia.span.immediately_follows(&token_span.into())
        // {
        //     self.scan_next_trivia();
        // }
    }

    pub fn scan_break(&mut self, space: bool) {
        self.last_break = Some(self.tokens.len());
        let len = if space { 1 } else { 0 };
        self.tokens.push_back(Token::Break { space, len });
    }

    pub fn scan_begin(&mut self, delim: impl Into<Text>, mode: BreakMode) {
        self.scan_text(delim);
        self.begin_stack.push(self.tokens.len());
        self.tokens.push_back(Token::Begin { mode, len: 0 });
    }

    pub fn scan_end(&mut self, delim: impl Into<Text>) {
        let delim = delim.into();
        // Flush any trivia that appears before this token
        if let Some(span) = delim.span() {
            self.flush_trivia(span);
        }

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

        self.scan_text(delim);
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
            Token::Text(text) => {
                let should_print = matches!(
                    (text.mode(), content_break),
                    (TextMode::Always, _) | (TextMode::Break, true) | (TextMode::NoBreak, false)
                );
                if should_print {
                    self.output.push_str(text);
                    self.space -= text.len() as isize;
                }
            }
            Token::Break { space, len } => {
                if content_break || *len as isize >= self.space {
                    self.print_break();
                } else {
                    if *space {
                        self.output.push(' ');
                    }
                    self.space -= *len as isize;
                }
            }
            Token::Begin { mode, len, .. } => {
                let group_break = *len as isize >= self.space && *mode == BreakMode::Consistent;
                self.print_frames.push(PrintFrame {
                    group_break,
                    content_break: group_break,
                });
                self.indent += 1;
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
