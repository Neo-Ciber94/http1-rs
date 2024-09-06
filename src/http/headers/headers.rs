use core::str;

use private::Sealed;

use super::{
    entry::{Entry, EntryValue},
    HeaderName,
};

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
                let entry = self.entries.get(idx)?;
                match &entry.value {
                    EntryValue::Single(x) => Some(x.as_str()),
                    EntryValue::List(list) => Some(list[0].as_str()),
                }
            }
            None => None,
        }
    }

    pub fn get_all(&self, key: impl AsHeaderName) -> GetAll {
        let iter = key
            .find(&self)
            .map(|idx| &self.entries[idx])
            .map(|x| x.iter());

        GetAll { iter }
    }

    pub fn get_mut(&mut self, key: impl AsHeaderName) -> Option<&mut String> {
        match key.find(&self) {
            Some(idx) => {
                let entry = self.entries.get_mut(idx)?;
                match &mut entry.value {
                    EntryValue::Single(x) => Some(x),
                    EntryValue::List(list) => Some(&mut list[0]),
                }
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
                let ret = std::mem::replace(&mut entry.value, EntryValue::Single(value.into()));
                match ret {
                    EntryValue::Single(x) => Some(x),
                    EntryValue::List(mut list) => Some(list.remove(0)),
                }
            }
            None => {
                self.entries.push(Entry {
                    key,
                    value: EntryValue::Single(value.into()),
                });
                None
            }
        }
    }

    pub fn append(&mut self, key: HeaderName, value: impl Into<String>) -> bool {
        match key.find(self) {
            Some(idx) => {
                let entry = &mut self.entries[idx];

                match &mut entry.value {
                    EntryValue::Single(x) => {
                        let cur = std::mem::take(x);
                        let list = vec![cur, value.into()];
                        entry.value = EntryValue::List(list);
                    }
                    EntryValue::List(list) => list.push(value.into()),
                }

                true
            }
            None => {
                self.entries.push(Entry {
                    key,
                    value: EntryValue::Single(value.into()),
                });
                false
            }
        }
    }

    pub fn remove(&mut self, key: impl AsHeaderName) -> Option<String> {
        match key.find(&self) {
            Some(idx) => {
                let entry = self.entries.remove(idx);
                Some(entry.take())
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
            entries: &self.entries,
            index: 0,
        }
    }

    pub fn into_iter(self) -> IntoIter {
        IntoIter {
            entries: self.entries,
        }
    }
}

pub struct GetAll<'a> {
    iter: Option<super::entry::Iter<'a>>,
}

impl<'a> Iterator for GetAll<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter {
            None => None,
            Some(ref mut x) => x.next(),
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
    entries: &'a Vec<Entry>,
    index: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a HeaderName, super::entry::Iter<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.entries.len() {
            return None;
        }

        let entry = self.entries.get(self.index)?;
        self.index += 1;
        Some((&entry.key, entry.iter()))
    }
}

impl<'a> IntoIterator for &'a Headers {
    type Item = (&'a HeaderName, super::entry::Iter<'a>);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IntoIter {
    entries: Vec<Entry>,
}

impl Iterator for IntoIter {
    type Item = (HeaderName, super::entry::IntoIter);

    fn next(&mut self) -> Option<Self::Item> {
        if self.entries.is_empty() {
            return None;
        }

        let entry = self.entries.remove(0);
        let key = entry.key.clone();
        Some((key, entry.into_iter()))
    }
}

impl IntoIterator for Headers {
    type Item = (HeaderName, super::entry::IntoIter);
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

        let (numbers_name, numbers) = iter.next().unwrap();
        assert_eq!(numbers_name.as_str(), "numbers");
        assert_eq!(
            numbers.collect::<Vec<&str>>(),
            vec!["1".to_owned(), "2".to_owned(), "3".to_owned()]
        );

        let (fruits_name, fruits) = iter.next().unwrap();
        assert_eq!(fruits_name.as_str(), "fruits");
        assert_eq!(
            fruits.collect::<Vec<&str>>(),
            vec!["apple".to_owned(), "strawberry".to_owned()]
        );

        let (food_name, foods) = iter.next().unwrap();
        assert_eq!(food_name.as_str(), "food");
        assert_eq!(foods.collect::<Vec<&str>>(), vec!["pizza".to_owned()]);

        assert!(iter.next().is_none())
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

        let (numbers_name, numbers) = iter.next().unwrap();
        assert_eq!(numbers_name.as_str(), "numbers");
        assert_eq!(
            numbers.collect::<Vec<String>>(),
            vec!["1".to_owned(), "2".to_owned(), "3".to_owned()]
        );

        let (fruits_name, fruits) = iter.next().unwrap();
        assert_eq!(fruits_name.as_str(), "fruits");
        assert_eq!(
            fruits.collect::<Vec<String>>(),
            vec!["apple".to_owned(), "strawberry".to_owned()]
        );

        let (food_name, foods) = iter.next().unwrap();
        assert_eq!(food_name.as_str(), "food");
        assert_eq!(foods.collect::<Vec<String>>(), vec!["pizza".to_owned()]);

        assert!(iter.next().is_none());
    }
}
