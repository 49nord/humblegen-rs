// From: https://users.rust-lang.org/t/iterator-need-to-identify-the-last-element/8836/10

//! This module contains general helper traits.
use std::{iter, mem};

/// This trait defines the iterator adapter `identify_first_last()`.
/// The new iterator gives a tuple with an `(element, is_first, is_last)`.
/// `is_first` is true when `element` is the first we are iterating over.
/// `is_last` is true when `element` is the last and no others follow.
pub trait IdentifyFirstLast: Iterator + Sized {
    fn identify_first_last(self) -> Iter<Self>;
}

/// Implement the iterator adapter `identify_first_last()`
impl<I> IdentifyFirstLast for I
where
    I: Iterator,
{
    fn identify_first_last(self) -> Iter<Self> {
        Iter(true, self.peekable())
    }
}

/// A struct to hold the iterator's state
/// Our state is a bool telling if this is the first element.
pub struct Iter<I>(bool, iter::Peekable<I>)
where
    I: Iterator;

impl<I> Iterator for Iter<I>
where
    I: Iterator,
{
    type Item = (bool, bool, I::Item);

    /// At `next()` we copy false to the state variable.
    /// And `peek()` adhead to see if this is the last one.
    fn next(&mut self) -> Option<Self::Item> {
        let first = mem::replace(&mut self.0, false);
        self.1.next().map(|e| (first, self.1.peek().is_none(), e))
    }
}
