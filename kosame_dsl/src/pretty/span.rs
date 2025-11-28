use proc_macro2::LineColumn;

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Span {
    start: LineColumn,
    end: LineColumn,
}

impl Span {
    #[must_use]
    pub fn new(start: LineColumn, end: LineColumn) -> Self {
        Self { start, end }
    }

    /// Returns true if this span immediately follows another span (no gap between them).
    #[must_use]
    pub fn immediately_follows(&self, other: &Span) -> bool {
        self.start.line == other.end.line && self.start.column == other.end.column
    }

    /// Returns true if this span comes before the given token span.
    #[must_use]
    pub fn comes_before(&self, other: &Span) -> bool {
        self.end.line < other.start.line
            || (self.end.line == other.start.line && self.end.column <= other.start.column)
    }

    #[must_use]
    pub fn start(&self) -> LineColumn {
        self.start
    }

    #[must_use]
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
