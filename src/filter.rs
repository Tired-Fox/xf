use std::fmt::Debug;

use crate::Entry;

pub trait Filter
{
    fn keep(&self, entry: &Entry) -> bool;

    #[inline]
    fn discard(&self, entry: &Entry) -> bool {
        !self.keep(entry)
    }
}

pub trait Binary
where
    Self: Sized
{
    type Not: Filter;

    fn and<B>(self, other: B) -> And<Self, B>;
    fn or<B>(self, other: B) -> Or<Self, B>;
    fn not(self) -> Self::Not;
}

impl<F: Filter> Binary for F {
    type Not = Not<Self>;

    fn and<B>(self, other: B) -> And<Self, B> {
        And(self, other)
    }

    fn or<B>(self, other: B) -> Or<Self, B> {
        Or(self, other)
    }

    fn not(self) -> Self::Not {
        Not(self)
    }
}

impl Filter for () {
    #[inline]
    fn keep(&self, _entry: &Entry) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Extensions {
    extensions: Vec<String>,
    case_sensitive: bool,
}

impl Extensions {
    pub fn new<I: IntoIterator<Item=S>, S: ToString>(extensions: I) -> Self {
        Self {
            extensions: extensions.into_iter().map(|v| v.to_string()).collect(),
            case_sensitive: false
        }
    }

    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }
}

impl Filter for Extensions {
    #[inline]
    fn keep(&self, entry: &Entry) -> bool {
        let ext = entry.extension().map(|v| if self.case_sensitive { v } else { v.to_ascii_lowercase() }).unwrap_or_default();
        self.extensions.contains(&ext)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dot;

impl Filter for Dot {
    fn keep(&self, entry: &Entry) -> bool {
        entry.is_dot() 
    }
}

#[derive(Debug, Clone)]
pub struct Match(regex::Regex);

impl Match {
    pub fn new<S: AsRef<str>>(pattern: S) -> Result<Self, regex::Error> {
        Ok(Self(regex::Regex::new(pattern.as_ref())?))
    }
}

impl Filter for Match {
    fn keep(&self, entry: &Entry) -> bool {
        self.0.is_match(entry.file_name()) 
    }
}

pub struct And<A, B>(A, B);

impl<A: Default, B: Default> Default for And<A, B> {
    fn default() -> Self {
        And(A::default(), B::default())
    }
}

impl<A: Debug, B: Debug> Debug for And<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chain")
            .field("A", &self.0)
            .field("B", &self.1)
            .finish()
    }
}

impl<A: PartialEq, B: PartialEq> PartialEq for And<A, B> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0) && self.1.eq(&other.1)
    }
}

impl<A: Clone, B: Clone> Clone for And<A, B> {
    fn clone(&self) -> Self {
        And(self.0.clone(), self.1.clone())
    }
}

impl<A, B> And<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self(a, b)
    }

    pub fn a(&self) -> &A {
        &self.0
    }

    pub fn b(&self) -> &B {
        &self.1
    }
}

impl<A: Filter, B: Filter> Filter for And<A, B> {
    fn keep(&self, entry: &Entry) -> bool {
        self.0.keep(entry) && self.1.keep(entry)
    }
}

pub struct Or<A, B>(A, B);

impl<A: Default, B: Default> Default for Or<A, B> {
    fn default() -> Self {
        Or(A::default(), B::default())
    }
}

impl<A: Debug, B: Debug> Debug for Or<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Or")
            .field("A", &self.0)
            .field("B", &self.1)
            .finish()
    }
}

impl<A: PartialEq, B: PartialEq> PartialEq for Or<A, B> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0) && self.1.eq(&other.1)
    }
}

impl<A: Clone, B: Clone> Clone for Or<A, B> {
    fn clone(&self) -> Self {
        Or(self.0.clone(), self.1.clone())
    }
}

impl<A, B> Or<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self(a, b)
    }

    pub fn a(&self) -> &A {
        &self.0
    }

    pub fn b(&self) -> &B {
        &self.1
    }
}

impl<A: Filter, B: Filter> Filter for Or<A, B> {
    fn keep(&self, entry: &Entry) -> bool {
        self.0.keep(entry) || self.1.keep(entry)
    }
}

pub struct Not<F>(F);

impl<F: Default> Default for Not<F> {
    fn default() -> Self {
        Not(F::default())
    }
}

impl<F: Debug> Debug for Not<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Not")
            .field("filter", &self.0)
            .finish()
    }
}

impl<F: PartialEq> PartialEq for Not<F> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<F: Clone> Clone for Not<F> {
    fn clone(&self) -> Self {
        Not(self.0.clone())
    }
}

impl<F> Not<F> {
    pub fn new(filter: F) -> Self {
        Self(filter)
    }

    pub fn filter(&self) -> &F {
        &self.0
    }
}

impl<F: Filter> Filter for Not<F> {
    fn keep(&self, entry: &Entry) -> bool {
        self.0.discard(entry)
    }
}
