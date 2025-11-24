use std::{
    collections::VecDeque,
    ops::{Index, IndexMut},
};

/// Lightweight abstraction over [`VecDeque`] that keeps index values after values are popped from
/// the front of the buffer.
pub struct RingBuffer<T> {
    inner: VecDeque<T>,
    offset: usize,
}

impl<T> RingBuffer<T> {
    pub fn new() -> Self {
        Self {
            inner: VecDeque::new(),
            offset: 0,
        }
    }

    pub fn push_back(&mut self, value: T) {
        self.inner.push_back(value);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.offset += 1;
        self.inner.pop_front()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<T> Index<usize> for RingBuffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index - self.offset]
    }
}

impl<T> IndexMut<usize> for RingBuffer<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index - self.offset]
    }
}
