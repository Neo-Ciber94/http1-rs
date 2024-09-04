use core::str;

use private::Sealed;

use super::HeaderName;

#[derive(Debug, Clone)]
struct Entry {
    key: HeaderName,
    value: String,

    // If the header old more than 1 value, is stored here
    next: Vec<String>,
}

#[derive(Default, Debug, Clone)]
pub struct Headers {
    entries: Vec<Entry>,
}

impl Headers {
    pub fn new() -> Self {
        Headers {
            ..Default::default()
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, key: impl AsHeaderName) -> Option<&str> {
        match key.find(&self) {
            Some(idx) => {
                let entry = &self.entries[idx];
                Some(&entry.value)
            }
            None => None,
        }
    }

    pub fn get_all(&self, key: impl AsHeaderName) -> GetAll {
        GetAll {
            entry: key.find(&self).map(|idx| &self.entries[idx]),
            index: None,
        }
    }

    pub fn get_mut(&mut self, key: impl AsHeaderName) -> Option<&mut String> {
        match key.find(&self) {
            Some(idx) => {
                let entry = &mut self.entries[idx];
                Some(&mut entry.value)
            }
            None => None,
        }
    }

    pub fn contains_key(&self, key: impl AsHeaderName) -> bool {
        key.find(self).is_some()
    }

    pub fn insert(&mut self, key: HeaderName, value: impl Into<String>) -> Option<String> {
        match key.find(self) {
            Some(idx) => {
                let entry = &mut self.entries[idx];
                let prev = std::mem::replace(&mut entry.value, value.into());
                Some(prev)
            }
            None => {
                self.entries.push(Entry {
                    key,
                    value: value.into(),
                    next: Vec::new(),
                });
                None
            }
        }
    }

    pub fn append(&mut self, key: HeaderName, value: impl Into<String>) -> bool {
        match key.find(self) {
            Some(idx) => {
                let entry = &mut self.entries[idx];
                entry.next.push(value.into());
                true
            }
            None => {
                self.entries.push(Entry {
                    key,
                    value: value.into(),
                    next: Vec::new(),
                });
                false
            }
        }
    }

    pub fn remove(&mut self, key: impl AsHeaderName) -> Option<String> {
        match key.find(&self) {
            Some(idx) => {
                let entry = self.entries.remove(idx);
                Some(entry.value)
            }
            None => None,
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn keys(&self) -> Keys {
        Keys {
            iter: self.entries.iter(),
        }
    }

    pub fn iter(&self) -> Iter {
        Iter {
            headers: self,
            entry_index: 0,
            pos: None,
        }
    }

    pub fn into_iter(self) -> IntoIter {
        IntoIter { headers: self }
    }
}

pub struct GetAll<'a> {
    entry: Option<&'a Entry>,
    index: Option<usize>,
}

impl<'a> Iterator for GetAll<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self.entry {
            None => None,
            Some(entry) => match self.index {
                None => {
                    self.index = Some(0);
                    Some(entry.value.as_str())
                }
                Some(idx) => {
                    let next = entry.next.get(idx)?;
                    self.index = Some(idx + 1);
                    Some(next.as_str())
                }
            },
        }
    }
}

pub struct Keys<'a> {
    iter: std::slice::Iter<'a, Entry>,
}

impl<'a> Iterator for Keys<'a> {
    type Item = &'a HeaderName;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| &x.key)
    }
}

pub struct Iter<'a> {
    headers: &'a Headers,
    entry_index: usize,
    pos: Option<usize>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a HeaderName, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.headers.is_empty() {
            return None;
        }

        while let Some(entry) = self.headers.entries.get(self.entry_index) {
            match self.pos {
                Some(pos) => match entry.next.get(pos) {
                    Some(value) => {
                        self.pos = Some(pos + 1);
                        return Some((&entry.key, value.as_str()));
                    }
                    None => {
                        // Move to the next entry
                        self.pos = None;
                        self.entry_index += 1;
                    }
                },
                None => {
                    self.pos = Some(0);
                    return Some((&entry.key, entry.value.as_str()));
                }
            }
        }

        None
    }
}

impl<'a> IntoIterator for &'a Headers {
    type Item = (&'a HeaderName, &'a str);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct EntryValues {
    entry: Entry,
    iter_next: bool,
}

impl Iterator for EntryValues {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = &mut self.entry;

        match self.iter_next {
            true if entry.next.is_empty() => None,
            true => {
                let item = entry.next.remove(0);
                Some(item)
            }
            false => {
                let value = std::mem::take(&mut entry.value);
                self.iter_next = true;
                Some(value)
            }
        }
    }
}

pub struct IntoIter {
    headers: Headers,
}

impl Iterator for IntoIter {
    type Item = (HeaderName, EntryValues);

    fn next(&mut self) -> Option<Self::Item> {
        let entries = &mut self.headers.entries;

        if entries.is_empty() {
            return None;
        }

        let entry = entries.remove(0);
        let key = entry.key.clone();
        Some((
            key,
            EntryValues {
                entry,
                iter_next: false,
            },
        ))
    }
}

impl IntoIterator for Headers {
    type Item = (HeaderName, EntryValues);
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

pub trait AsHeaderName: private::Sealed {}

impl AsHeaderName for String {}
impl AsHeaderName for HeaderName {}
impl<'a> AsHeaderName for &'a HeaderName {}
impl<'a> AsHeaderName for &'a str {}
impl<'a> AsHeaderName for &'a String {}

impl<'a> private::Sealed for &'a str {
    fn find(&self, map: &Headers) -> Option<usize> {
        map.entries
            .iter()
            .position(|entry| entry.key.as_str().eq_ignore_ascii_case(self))
    }
}

impl<'a> private::Sealed for &'a String {
    fn find(&self, map: &Headers) -> Option<usize> {
        private::Sealed::find(&self.as_str(), map)
    }
}

impl private::Sealed for String {
    fn find(&self, map: &Headers) -> Option<usize> {
        private::Sealed::find(&self.as_str(), map)
    }
}

impl<'a> private::Sealed for &'a HeaderName {
    fn find(&self, map: &Headers) -> Option<usize> {
        private::Sealed::find(&self.as_str(), map)
    }
}

impl private::Sealed for HeaderName {
    fn find(&self, map: &Headers) -> Option<usize> {
        private::Sealed::find(&self.as_str(), map)
    }
}

mod private {
    use super::Headers;

    pub trait Sealed {
        fn find(&self, map: &Headers) -> Option<usize>;
    }
}

#[cfg(test)]
mod tests {
    use crate::http::headers::HeaderName;

    use super::Headers;

    #[test]
    fn should_insert() {
        let mut headers = Headers::new();
        headers.insert("Accept".into(), "application/json");
        headers.insert("Content Length".into(), "120");

        assert_eq!(headers.len(), 2)
    }

    #[test]
    fn should_be_case_insensitive() {
        let mut headers = Headers::new();
        headers.insert("abc".into(), "1");
        headers.insert("xyz".into(), "2");
        headers.insert("JKL".into(), "3");

        assert_eq!(headers.get("ABC"), Some("1"));
        assert_eq!(headers.get("XyZ"), Some("2"));
        assert_eq!(headers.get("jKl"), Some("3"));
    }

    #[test]
    fn should_get_mut() {
        let mut headers = Headers::new();
        headers.insert("Accept".into(), "hello");
        headers.get_mut("accept").unwrap().push_str("-world");

        assert_eq!(headers.get("accept"), Some("hello-world"));
    }

    #[test]
    fn should_get_all() {
        let mut headers = Headers::new();
        headers.append("Fruits".into(), "apple");
        headers.append("Fruits".into(), "strawberry");
        headers.append("Fruits".into(), "banana");

        assert_eq!(headers.get("fruits"), Some("apple"));
        assert_eq!(
            headers.get_all("fruits").collect::<Vec<&str>>(),
            vec!["apple", "strawberry", "banana"]
        );
    }

    #[test]
    fn should_replace_existing_on_insert() {
        let mut headers = Headers::new();
        headers.insert("Accept".into(), "text/html");
        headers.insert("Accept-Encoding".into(), "gzip");

        headers.insert("accept".into(), "text/plain");
        headers.insert("accept-encoding".into(), "br");

        assert_eq!(headers.get("accept"), Some("text/plain"));
        assert_eq!(headers.get("accept-encoding"), Some("br"))
    }

    #[test]
    fn should_remove() {
        let mut headers = Headers::new();
        headers.insert("abc".into(), "1");
        headers.insert("xyz".into(), "2");
        headers.insert("JKL".into(), "3");

        assert_eq!(headers.remove("abc"), Some("1".to_owned()));
        assert_eq!(headers.remove("abc"), None);

        assert_eq!(headers.remove("xYz"), Some("2".to_owned()));
        assert_eq!(headers.remove("xYz"), None);

        assert_eq!(headers.remove("jKL"), Some("3".to_owned()));
        assert_eq!(headers.remove("jKL"), None);
    }

    #[test]
    fn should_iter_over_all_entries() {
        let mut headers = Headers::new();
        headers.append("numbers".into(), "1");
        headers.append("numbers".into(), "2");
        headers.append("numbers".into(), "3");
        headers.append("fruits".into(), "apple");
        headers.append("fruits".into(), "strawberry");
        headers.append("food".into(), "pizza");

        let mut iter = headers.iter();

        assert_eq!(iter.next(), Some((&HeaderName::from("numbers"), "1")));
        assert_eq!(iter.next(), Some((&HeaderName::from("numbers"), "2")));
        assert_eq!(iter.next(), Some((&HeaderName::from("numbers"), "3")));

        assert_eq!(iter.next(), Some((&HeaderName::from("fruits"), "apple")));
        assert_eq!(
            iter.next(),
            Some((&HeaderName::from("fruits"), "strawberry"))
        );

        assert_eq!(iter.next(), Some((&HeaderName::from("food"), "pizza")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn should_into_iter_over_all_entries() {
        let mut headers = Headers::new();
        headers.append("numbers".into(), "1");
        headers.append("numbers".into(), "2");
        headers.append("numbers".into(), "3");
        headers.append("fruits".into(), "apple");
        headers.append("fruits".into(), "strawberry");
        headers.append("food".into(), "pizza");

        let mut iter = headers.into_iter();

        let numbers = iter.next().unwrap();
        assert_eq!(numbers.0.as_str(), "numbers");
        assert_eq!(
            numbers.1.collect::<Vec<String>>(),
            vec!["1".to_owned(), "2".to_owned(), "3".to_owned()]
        );

        let fruits = iter.next().unwrap();
        assert_eq!(fruits.0.as_str(), "fruits");
        assert_eq!(
            fruits.1.collect::<Vec<String>>(),
            vec!["apple".to_owned(), "strawberry".to_owned()]
        );

        let food = iter.next().unwrap();
        assert_eq!(food.0.as_str(), "food");
        assert_eq!(food.1.collect::<Vec<String>>(), vec!["pizza".to_owned()]);

        assert!(iter.next().is_none());
    }
}
