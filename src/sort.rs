use std::cmp::Ordering;

use crate::{Entry, IsHidden};

/// Helper to determine state of a char from an iterator
pub trait IterChar {
    fn is_ascii_digit(&self) -> bool;
}
impl IterChar for Option<&char> {
    fn is_ascii_digit(&self) -> bool {
        self.map(|v| v.is_ascii_digit()).unwrap_or_default()
    }
}
impl IterChar for Option<char> {
    fn is_ascii_digit(&self) -> bool {
        self.map(|v| v.is_ascii_digit()).unwrap_or_default()
    }
}

/// Implement to allow a struct be a sorter for [`crate::Entry`]
pub trait SortStrategy {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering;
}

// Default sorter sorts by comparing file names as strings
impl SortStrategy for () {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering {
        let first = first.as_entry().path();
        let second = second.as_entry().path();
        match first.cmp(&second) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            other => other
        }
    }
}

/// A sorter that will sort hidden files first
pub struct Hidden<T = Natural>(T);
impl<T: Default> Default for Hidden<T> {
    fn default() -> Self {
        Hidden(T::default())
    }
}
impl<T: PartialEq> PartialEq for Hidden<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<T: SortStrategy> SortStrategy for Hidden<T> {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering {
        match (first.as_entry().is_hidden(), second.as_entry().is_hidden()) {
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            _ => self.0.compare(first, second)
        }
    }
}

/// Sorter that implements the Natrual sort order (Human sort) algorithm.
///
/// It will treat numbers as numbers. So if two paths have number in the same position in the name
/// then the numbers are parsed and compared. All other characters are compared as regular
/// characters.
///
/// # Example
///
/// ```plaintext
/// _2.txt
/// _12.txt
/// _1.txt
/// ```
///
/// Will be sorted as
///
/// ```plaintext
/// _1.txt
/// _2.txt
/// _12.txt
/// ````
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Natural;
impl SortStrategy for Natural {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering {
        // ab102c -> a b 102 c
        // ab20a -> a b 20 a
        let first = first.file_name().to_string_lossy().to_string();
        let mut first = first.chars().peekable();
        let second = second.file_name().to_string_lossy().to_string();
        let mut second = second.chars().peekable();

        while let (Some(_), Some(_)) = (first.peek(), second.peek()) {
            if first.peek().is_ascii_digit() && second.peek().is_ascii_digit() {
                let u = first.clone().take_while(|v| v.is_ascii_digit()).collect::<String>().parse::<usize>().unwrap();
                let v = second.clone().take_while(|v| v.is_ascii_digit()).collect::<String>().parse::<usize>().unwrap();

                match u.cmp(&v) {
                    Ordering::Less => return Ordering::Greater,
                    Ordering::Greater => return Ordering::Less,
                    _ => {}
                }
            } else {
                // If comparison is not equal return it immediatly
                match first.next().unwrap().cmp(&second.next().unwrap()) {
                    Ordering::Less => return Ordering::Greater,
                    Ordering::Greater => return Ordering::Less,
                    _ => {}
                }
            }
        }

        match (first.peek(), second.peek()) {
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            _ => Ordering::Equal
        }
    }
}

/// A sorter that will sort directories first
pub struct Directory<T = Natural>(T);
impl<T: Default> Default for Directory<T> {
    fn default() -> Self {
        Directory(T::default())
    }
}
impl<T: PartialEq> PartialEq for Directory<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<T: SortStrategy> SortStrategy for Directory<T> {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering {
        match (first, second) {
            (Entry::Dir(_), Entry::File(_)) => Ordering::Greater,
            (Entry::File(_), Entry::Dir(_)) => Ordering::Less,
            _ => {
                self.0.compare(first, second)
            }
        }
    }
}

pub trait Matches {
    fn matches(entry: &Entry) -> bool;
}

impl<T> Matches for Hidden<T> {
    fn matches(entry: &Entry) -> bool {
        entry.as_entry().is_hidden()
    }
}

impl<T> Matches for Directory<T> {
    fn matches(entry: &Entry) -> bool {
        entry.as_entry().path().is_dir()
    }
}

impl Matches for Natural {
    fn matches(_entry: &Entry) -> bool {
        true
    }
}

impl Matches for () {
    fn matches(_entry: &Entry) -> bool {
        true
    }
}

impl<T> Matches for Extension<T> {
    fn matches(entry: &Entry) -> bool {
        entry.extension().is_some()
    }
}

// Sort by file extension
pub struct Extension<T = Natural>(T);
impl<T: Default> Default for Extension<T> {
    fn default() -> Self {
        Extension(T::default())
    }
}
impl<T: PartialEq> PartialEq for Extension<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<T: SortStrategy> SortStrategy for Extension<T> {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering {
        match (first.extension(), second.extension()) {
            (Some(f), Some(s)) => match f.cmp(&s) {
                Ordering::Less => Ordering::Greater,
                Ordering::Greater => Ordering::Less,
                Ordering::Equal => self.0.compare(first, second)
            },
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (None, None) => self.0.compare(first, second)
        }
    }
}

pub trait Grouping<T = ()> {
    fn get_group_index(entry: &Entry) -> Option<usize>;
    fn compare_within_group(&self, index: usize, first: &Entry, second: &Entry) -> Ordering;
}

impl<T1, T2> Grouping for (T1, T2)
where
    T1: SortStrategy + Matches,
    T2: SortStrategy + Matches,
{
    fn get_group_index(entry: &Entry) -> Option<usize> {
        if T1::matches(entry) { Some(0) }
        else if T2::matches(entry) { Some(1) }
        else { None }
    }

    fn compare_within_group(&self, index: usize, first: &Entry, second: &Entry) -> Ordering {
        match index {
            0 => self.0.compare(first, second),
            1 => self.1.compare(first, second),
            _ => Ordering::Equal
        }
    }
}

pub struct Group<T, D = Natural>(T, D);
impl<T: Default, D: Default> Default for Group<T, D> {
    fn default() -> Self {
        Group(T::default(), D::default())
    }
}
impl<T: Grouping, D: SortStrategy> SortStrategy for Group<T, D> {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering {
        let f = T::get_group_index(first);
        let s = T::get_group_index(second);

        match (f, s) {
            (Some(f), Some(s)) => match f.cmp(&s) {
                Ordering::Less => Ordering::Greater,
                Ordering::Greater => Ordering::Less,
                Ordering::Equal => self.0.compare_within_group(f, first, second)
            },
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (None, None) => self.1.compare(first, second)
        }
    }
}
