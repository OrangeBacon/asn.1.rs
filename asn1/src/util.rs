use std::{collections::VecDeque, ops::Deref};

/// Iterator extension trait for peekable iterators
pub trait Peek: Iterator
where
    Self: Sized,
{
    /// Create a peekable version of the iterator
    fn n_peekable(self) -> Peekable<Self>;
}

/// Implementation struct for peekable
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Peekable<I: Iterator> {
    /// Source iterator
    iter: I,

    /// Cached items peeked but not output
    cache: VecDeque<I::Item>,
}

impl<I: Iterator> Peek for I {
    /// Create an iterator that can peek unlimited items ahead of the iterator
    fn n_peekable(self) -> Peekable<Self> {
        Peekable {
            iter: self,
            cache: VecDeque::new(),
        }
    }
}

impl<I: Iterator> Iterator for Peekable<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.cache.pop_front().or_else(|| self.iter.next())
    }
}

impl<I: Iterator> Peekable<I> {
    /// Peek n tokens ahead of the iterator.  Peek with n == 0 returns the same
    /// item that `iter.next()` would return, but without consuming it.
    pub fn peek(&mut self, n: usize) -> Option<&I::Item> {
        loop {
            if self.cache.len() > n {
                break;
            }
            self.cache.push_back(self.iter.next()?);
        }

        Some(&self.cache[n])
    }
}

/// Version of std's cow specialised for slices
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CowVec<T: 'static> {
    Borrowed(&'static [T]),
    Owned(Vec<T>),
    Single(T),
}

impl<T> From<Vec<T>> for CowVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self::Owned(value)
    }
}

impl<T> From<&'static [T]> for CowVec<T> {
    fn from(value: &'static [T]) -> Self {
        Self::Borrowed(value)
    }
}

impl<T> From<&'static mut [T]> for CowVec<T> {
    fn from(value: &'static mut [T]) -> Self {
        Self::Borrowed(value)
    }
}

impl<const N: usize, T> From<&'static [T; N]> for CowVec<T> {
    fn from(value: &'static [T; N]) -> Self {
        Self::Borrowed(value)
    }
}

impl<const N: usize, T> From<&'static mut [T; N]> for CowVec<T> {
    fn from(value: &'static mut [T; N]) -> Self {
        Self::Borrowed(value)
    }
}

impl<T> From<T> for CowVec<T> {
    fn from(value: T) -> Self {
        CowVec::Single(value)
    }
}

impl<T> Deref for CowVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match self {
            CowVec::Borrowed(b) => b,
            CowVec::Owned(o) => o,
            CowVec::Single(v) => std::slice::from_ref(v),
        }
    }
}

impl<T> Default for CowVec<T> {
    fn default() -> Self {
        Self::Borrowed(&[])
    }
}

impl<T: Copy> CowVec<T> {
    /// Append a new vec to this vec, consuming both.
    pub fn append(self, value: impl Into<Self>) -> Self {
        let value = value.into();

        if self.is_empty() {
            value
        } else {
            CowVec::Owned(self.iter().chain(&*value).copied().collect())
        }
    }
}
