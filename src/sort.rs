use std::cmp::Ordering;

use chrono::Local;

use crate::{Directory, Entry, Hidden};

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
impl IterChar for &str {
    fn is_ascii_digit(&self) -> bool {
        self.len() == 1 && self.chars().nth(0).is_ascii_digit()
    }
}
impl IterChar for str {
    fn is_ascii_digit(&self) -> bool {
        self.len() == 1 && self.chars().nth(0).is_ascii_digit()
    }
}

/// Implement to allow a struct be a sorter for [`crate::Entry`]
pub trait SortStrategy {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering;
}

// Default sorter sorts by comparing file names as strings
impl SortStrategy for () {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering {
        first.path().cmp(second.path())
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
        let mut i = 0usize;
        let mut j =  0usize;

        let first = first.file_name();
        let second = second.file_name();

        let _ = second[j..j+1];
        while i < first.len() && j < second.len() {
            if first[i..i+1].is_ascii_digit() && second[j..j+1].is_ascii_digit() {
                let u = i; 
                let v = j;
                while i < first.len() && first[i..i+1].is_ascii_digit() {
                    i+=1;
                }
                while j < second.len() && second[j..j+1].is_ascii_digit() {
                    j+=1;
                }

                let u = first[u..i].parse::<usize>().unwrap();
                let v = second[v..j].parse::<usize>().unwrap();

                match u.cmp(&v) {
                    Ordering::Equal => {},
                    other => return other,
                }
            } else {
                // If comparison is not equal return it immediatly
                match first[i..i+1].cmp(&second[j..j+1]) {
                    Ordering::Equal => {},
                    other => return other,
                }
            }
            i += 1;
            j += 1;
        }

        match (i < first.len(), j < second.len()) {
            (false, true) => Ordering::Less,
            (true, false) => Ordering::Greater,
            _ => Ordering::Equal
        }
    }
}

pub trait Matches {
    fn matches(entry: &Entry) -> bool;
}

impl<T> Matches for Hidden<T> {
    fn matches(entry: &Entry) -> bool {
        entry.is_hidden()
    }
}

impl<T> Matches for Directory<T> {
    fn matches(entry: &Entry) -> bool {
        entry.is_dir()
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
        Self(T::default())
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
                Ordering::Equal => self.0.compare(first, second),
                other => other
            },
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
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
        Self(T::default(), D::default())
    }
}
impl<T: Grouping, D: SortStrategy> SortStrategy for Group<T, D> {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering {
        let f = T::get_group_index(first);
        let s = T::get_group_index(second);

        match (f, s) {
            (Some(f), Some(s)) => match f.cmp(&s) {
                Ordering::Equal => self.0.compare_within_group(f, first, second),
                other => other,
            },
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (None, None) => self.1.compare(first, second)
        }
    }
}

pub struct Date<T=Natural>(T);
impl Default for Date {
    fn default() -> Self {
        Self(Natural)
    }
}
impl<T: SortStrategy> SortStrategy for Date<T> {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering {
        let f: Option<chrono::DateTime<Local>> = first.metadata().modified().map(|t| t.into()).ok();
        let s: Option<chrono::DateTime<Local>> = second.metadata().modified().map(|t| t.into()).ok();

        match (f, s) {
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(f), Some(s)) => f.date_naive().cmp(&s.date_naive()),
            (None, None) => self.0.compare(first, second)
        }
    }
}

pub struct Time<T=Natural>(T);
impl Default for Time {
    fn default() -> Self {
        Self(Natural)
    }
}
impl<T: SortStrategy> SortStrategy for Time<T> {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering {
        let f: Option<chrono::DateTime<Local>> = first.metadata().modified().map(|t| t.into()).ok();
        let s: Option<chrono::DateTime<Local>> = second.metadata().modified().map(|t| t.into()).ok();

        match (f, s) {
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(f), Some(s)) => f.time().cmp(&s.time()),
            (None, None) => self.0.compare(first, second)
        }
    }
}

pub struct DateTime<T=Natural>(T);
impl Default for DateTime {
    fn default() -> Self {
        Self(Natural)
    }
}
impl<T: SortStrategy> SortStrategy for DateTime<T> {
    fn compare(&self, first: &Entry, second: &Entry) -> Ordering {
        let f: Option<chrono::DateTime<Local>> = first.metadata().modified().map(|t| t.into()).ok();
        let s: Option<chrono::DateTime<Local>> = second.metadata().modified().map(|t| t.into()).ok();

        match (f, s) {
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(f), Some(s)) => f.cmp(&s),
            (None, None) => self.0.compare(first, second)
        }
    }
}
