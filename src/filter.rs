use crate::Entry;

pub trait Filter {
    fn keep(&self, entry: &Entry) -> bool;

    #[inline]
    fn discard(&self, entry: &Entry) -> bool {
        !self.keep(entry)
    }
}

impl Filter for () {
    #[inline]
    fn keep(&self, _entry: &Entry) -> bool {
        true
    }
}

pub struct Extensions(pub Vec<String>);
impl Filter for Extensions {
    #[inline]
    fn keep(&self, entry: &Entry) -> bool {
        if entry.as_entry().path().is_dir() { return true }
        self.0.contains(&entry.extension().unwrap_or_default())
    }
}
