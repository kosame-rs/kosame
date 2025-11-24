use proc_macro2::LineColumn;

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Span {
    start: LineColumn,
    end: LineColumn,
}

impl Span {
    pub fn new(start: LineColumn, end: LineColumn) -> Self {
        Self { start, end }
    }

    pub fn file(source_text: &str) -> Self {
        let mut line = 1;
        let mut column = 0;
        for char in source_text.chars() {
            match char {
                '\n' => {
                    line += 1;
                    column = 0;
                }
                _ => column += 1,
            }
        }
        Self {
            start: LineColumn { line: 1, column: 0 },
            end: LineColumn { line, column },
        }
    }

    pub fn first() -> Self {
        Self {
            start: LineColumn { line: 1, column: 0 },
            end: LineColumn { line: 1, column: 1 },
        }
    }

    pub fn last(source_text: &str) -> Self {
        let mut line = 1;
        let mut column = 0;
        for char in source_text.chars() {
            match char {
                '\n' => {
                    line += 1;
                    column = 0;
                }
                _ => column += 1,
            }
        }
        Self {
            start: LineColumn { line, column },
            end: LineColumn {
                line,
                column: column + 1,
            },
        }
    }

    /// Returns true if this span immediately follows another span (no gap between them).
    pub fn immediately_follows(&self, other: &Span) -> bool {
        self.start.line == other.end.line && self.start.column == other.end.column
    }

    /// Returns true if this span comes before the given token span.
    pub fn comes_before(&self, other: &Span) -> bool {
        self.end.line < other.start.line
            || (self.end.line == other.start.line && self.end.column <= other.start.column)
    }

    pub fn start(&self) -> LineColumn {
        self.start
    }

    pub fn end(&self) -> LineColumn {
        self.end
    }
}

impl From<proc_macro2::Span> for Span {
    fn from(span: proc_macro2::Span) -> Self {
        Span {
            start: span.start(),
            end: span.end(),
        }
    }
}
