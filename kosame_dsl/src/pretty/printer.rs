use std::borrow::Cow;

use proc_macro2::LineColumn;

use crate::pretty::{
    BeginToken, BreakMode, BreakToken, TextMode, TextToken, Token, TokenBuffer, Trivia, TriviaKind,
};

pub const MARGIN: isize = 89;
pub const INDENT: isize = 4;
pub const MIN_SPACE: isize = 60;

#[derive(Debug)]
struct PrintFrame {
    group_break: bool,
}

pub struct Printer<'a> {
    trivia: &'a [Trivia<'a>],
    tokens: TokenBuffer<'a>,
    output: String,
    space: isize,
    scan_indent: isize,
    print_indent: isize,
    print_frames: Vec<PrintFrame>,
    cursor: LineColumn,
}

impl<'a> Printer<'a> {
    #[must_use]
    pub fn new(trivia: &'a [Trivia<'a>], initial_space: isize, initial_indent: isize) -> Self {
        Self {
            trivia,
            output: String::new(),
            space: initial_space.max(MIN_SPACE),
            scan_indent: initial_indent,
            print_indent: 0,
            tokens: TokenBuffer::new(),
            print_frames: Vec::new(),
            cursor: LineColumn { line: 1, column: 0 },
        }
    }

    pub fn move_cursor(&mut self, cursor: LineColumn) {
        self.cursor = cursor;
    }

    pub fn scan_text(&mut self, string: Cow<'static, str>, mode: TextMode) {
        self.tokens.push_len(string.len().try_into().unwrap());
        for char in string.chars() {
            match char {
                '\n' => {
                    self.cursor.line += 1;
                    self.cursor.column = 0;
                }
                _ => self.cursor.column += 1,
            }
        }
        let token = Token::Text(TextToken::new(string, mode));
        self.tokens.push_back(token);
    }

    pub fn scan_break(&mut self, force: bool) {
        let len = if force { MARGIN } else { 0 };
        self.tokens
            .push_back(Token::Break(BreakToken::new(len, self.scan_indent, force)));
        self.tokens.push_len(len);
    }

    pub fn scan_indent(&mut self, indent: isize) {
        self.scan_indent += indent;
    }

    pub fn scan_begin(&mut self, mode: BreakMode) {
        self.tokens
            .push_back(Token::Begin(BeginToken::new(mode, 0)));
    }

    /// # Panics
    ///
    /// Panics if there was no matching call to [`scan_begin`] prior to running this function.
    pub fn scan_end(&mut self) {
        let len = self
            .tokens
            .current_begin_mut()
            .expect("scanned end without matching begin")
            .len();
        self.tokens.push_back(Token::End);
        // Add child block length to parent.
        if let Some(parent) = self.tokens.current_begin_mut() {
            parent.push_len(len);
        }
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
        if !self.line_dirty() {
            string = string.trim_start();
        }
        self.print_indent();
        self.output.push_str(string);
        self.space -= isize::try_from(string.len()).unwrap();
    }

    fn print_break(&mut self) {
        self.output.push('\n');
        self.space = MARGIN;
    }

    fn print_indent(&mut self) {
        if !self.line_dirty() {
            self.output
                .push_str(&" ".repeat((self.print_indent * INDENT).try_into().unwrap()));
            self.space = (self.space - self.print_indent * INDENT).max(MIN_SPACE);
        }
    }

    fn print_first(&mut self) {
        let token = self.tokens.pop_front().expect("no tokens to print");

        let group_break = self
            .print_frames
            .last()
            .is_some_and(|frame| frame.group_break);

        match &token {
            Token::Text(text_token) => {
                let should_print = matches!(
                    (text_token.mode(), group_break),
                    (TextMode::Always, _) | (TextMode::Break, true) | (TextMode::NoBreak, false)
                );
                if should_print {
                    self.print_string(text_token.string());
                }
            }
            Token::Break(break_token) => {
                if group_break || break_token.len() >= self.space || break_token.force() {
                    self.print_break();
                    self.print_indent = break_token.indent();
                }
            }
            Token::Begin(begin_token) => {
                let group_break =
                    begin_token.len() >= self.space && begin_token.mode() == BreakMode::Consistent;
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
