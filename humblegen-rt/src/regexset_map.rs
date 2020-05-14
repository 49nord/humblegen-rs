//! `GEN,SERVER` - `RegexSetMap` maps a `s: &str` and some generic input `i: I` to an `Entry<I>`.
//!
//! An entry is a match candidate if
//!
//! - `.regex()` must match `s` and
//! - `.matches_input(i(` must return true
//!
//! The `GetResult` contains a reference to the matching entry.

use core::fmt;

/// Refer to module-level docs.
pub struct RegexSetMap<I, T: Entry<I>> {
    set: regex::RegexSet,
    entries: Vec<T>,
    _marker: std::marker::PhantomData<I>,
}

/// Refer to module-level docs.
pub trait Entry<I> {
    fn regex(&self) -> &regex::Regex;
    fn matches_input(&self, i: &I) -> bool;
}

impl<I, T: Entry<I>> Entry<I> for (regex::Regex, T) {
    fn regex(&self) -> &regex::Regex {
        &self.0
    }
    fn matches_input(&self, i: &I) -> bool {
        self.1.matches_input(i)
    }
}

impl<I, T: Entry<I>> fmt::Debug for RegexSetMap<I, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RegexSetMap")
            .field("set.patterns", &self.set.patterns())
            .field("entries", &"<NoTraitSpecialization>")
            .finish()
    }
}

/// Refer to module-level docs.
#[derive(Debug)]
pub enum GetResult<'a, T> {
    None,
    One(&'a T),
    Ambiguous,
}

impl<I, T: Entry<I>> RegexSetMap<I, T> {
    /// Refer to module-level docs.
    pub fn new(entries: Vec<T>) -> Result<Self, regex::Error> {
        let set = regex::RegexSet::new(entries.iter().map(|r| r.regex().as_str())).unwrap();
        Ok(Self {
            set,
            entries,
            _marker: std::marker::PhantomData::default(),
        })
    }

    /// Refer to module-level docs.
    pub fn get(&self, s: &str, input: &I) -> GetResult<'_, T> {
        let mut matching_route_idxs = self
            .set
            .matches(s)
            .into_iter()
            .filter(|matching_idx| self.entries[*matching_idx].matches_input(input))
            .peekable();

        let matching_route_idx = matching_route_idxs.next();
        let next_matching_route_idx = matching_route_idxs.peek();

        let matching_idx = match (matching_route_idx, next_matching_route_idx) {
            (Some(idx), None) => idx,
            (None, s @ Some(_)) => {
                unreachable!("peek after next() == None always returns None, got {:?}", s)
            }
            (None, None) => {
                return GetResult::None;
            }
            (Some(_), Some(_)) => {
                return GetResult::Ambiguous;
            }
        };

        GetResult::One(&self.entries[matching_idx])
    }
}
