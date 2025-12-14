pub trait Exec {
    type Response;

    fn exec(&self) -> Self::Response;
}

pub trait Response {}

pub struct RowCount(u64);

impl RowCount {
    #[inline]
    #[must_use]
    pub const fn new(row_count: u64) -> Self {
        Self(row_count)
    }

    #[must_use]
    const fn row_count(&self) -> u64 {
        self.0
    }
}

impl Response for RowCount {}
