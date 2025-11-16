use std::{borrow::Cow, collections::VecDeque};

const MARGIN: usize = 89;
const INDENT: usize = 4;
const MIN_SPACE: usize = 60;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BreakMode {
    Consistent,
    Inconsistent,
}

enum Token {
    Text(Cow<'static, str>),
    Break { text: Cow<'static, str>, len: usize },
    Begin { mode: BreakMode, len: usize },
    End,
}

impl Token {
    fn len(&self) -> usize {
        match self {
            Self::Text(inner) => inner.len(),
            Self::Break { len, .. } => *len,
            Self::Begin { len, .. } => *len,
            Self::End => 0,
        }
    }
}

struct SizedToken {
    inner: Token,
    size: i32,
}

#[derive(Default)]
pub struct Printer {
    output: String,
    space: isize,
    indent: usize,
    tokens: VecDeque<Token>,
    last_break: Option<usize>,
    begin_stack: Vec<usize>,
    force_break_stack: Vec<bool>,
}

impl Printer {
    pub fn new(initial_space: usize, initial_indent: usize) -> Self {
        Self {
            output: String::new(),
            space: initial_space as isize,
            indent: initial_indent,
            tokens: VecDeque::new(),
            last_break: None,
            begin_stack: Vec::new(),
            force_break_stack: Vec::new(),
        }
    }

    fn token(&self, index: usize) -> &Token {
        &self.tokens[index]
    }

    fn token_mut(&mut self, index: usize) -> &mut Token {
        &mut self.tokens[index]
    }

    pub fn print_text(&mut self, text: impl Into<Cow<'static, str>>) {
        let text = text.into();
        let text_len = text.len();
        self.tokens.push_back(Token::Text(text));

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

    pub fn print_break(&mut self, text: impl Into<Cow<'static, str>>) {
        let text = text.into();
        let len = text.len();
        self.last_break = Some(self.tokens.len());
        self.tokens.push_back(Token::Break { text, len });
    }

    pub fn print_begin(&mut self, mode: BreakMode) {
        self.begin_stack.push(self.tokens.len());
        self.tokens.push_back(Token::Begin { mode, len: 0 });
    }

    pub fn print_end(&mut self) {
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

    fn emit_first(&mut self) {
        let token = self.tokens.pop_front().expect("no tokens to emit");
        match &token {
            Token::Text(text) => {
                self.output.push_str(text);
            }
            Token::Break { text, len } => {
                let force_break = self.force_break_stack.last().copied().unwrap_or(false);
                if force_break || *len as isize >= self.space {
                    self.output.push('\n');
                    self.output.push_str(&" ".repeat(self.indent * INDENT));
                    self.space = (MARGIN - self.indent * INDENT).max(MIN_SPACE) as isize;
                } else {
                    self.output.push_str(text);
                }
            }
            Token::Begin { mode, len, .. } => {
                self.force_break_stack
                    .push(*len as isize >= self.space && *mode == BreakMode::Consistent);
                self.indent += 1;
            }
            Token::End => {
                self.force_break_stack.pop();
                self.indent -= 1;
            }
        };
        self.space -= token.len() as isize;
    }

    pub fn eof(mut self) -> String {
        while !self.tokens.is_empty() {
            self.emit_first();
        }

        self.output
    }
}

pub trait PrettyPrint {
    fn pretty_print(&self, printer: &mut Printer);
}
