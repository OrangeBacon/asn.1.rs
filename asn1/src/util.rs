use std::collections::VecDeque;

/// Iterator extension trait for peekable iterators
pub trait Peek
where
    Self: Iterator,
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
