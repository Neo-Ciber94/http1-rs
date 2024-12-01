use core::str;
use std::fmt::Debug;

use private::Sealed;

use super::{non_empty_list::NonEmptyList, value::HeaderValue, HeaderName};

#[derive(Debug, Clone)]
struct HeaderEntry {
    pub(crate) key: HeaderName,
    pub(crate) value: NonEmptyList<HeaderValue>,
}

#[derive(Default, Clone)]
pub struct Headers {
    entries: Vec<HeaderEntry>,
}

impl Debug for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct DebugHeaderValueList<'a>(crate::headers::non_empty_list::Iter<'a, HeaderValue>);
        impl Debug for DebugHeaderValueList<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let values = self.0.clone().map(|x| x.as_str());
                f.debug_list().entries(values).finish()
            }
        }

        let mut map = f.debug_map();

        for (key, mut values) in self {
            debug_assert!(values.len() > 0);

            // This shouldn't happen
            if values.len() == 0 {
                continue;
            }

            if values.len() == 1 {
                let header_value = values.next().unwrap();
                map.entry(&key.as_str(), &header_value.as_str());
            } else {
                map.entry(&key.as_str(), &DebugHeaderValueList(values));
            }
        }

        map.finish()
    }
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

    pub fn get(&self, key: impl AsHeaderName) -> Option<&HeaderValue> {
        match key.find(self) {
            Some(idx) => {
                let entry = self.entries.get(idx)?;
                entry.value.iter().next()
            }
            None => None,
        }
    }

    pub fn get_all(&self, key: impl AsHeaderName) -> GetAll {
        let iter = key
            .find(self)
            .map(|idx| &self.entries[idx])
            .map(|x| x.value.iter());

        GetAll { iter }
    }

    pub fn get_mut(&mut self, key: impl AsHeaderName) -> Option<&mut HeaderValue> {
        match key.find(self) {
            Some(idx) => {
                let entry = self.entries.get_mut(idx)?;
                entry.value.first_mut()
            }
            None => None,
        }
    }

    pub fn contains_key(&self, key: impl AsHeaderName) -> bool {
        key.find(self).is_some()
    }

    pub fn insert(
        &mut self,
        key: HeaderName,
        value: impl Into<HeaderValue>,
    ) -> Option<HeaderValue> {
        let value = NonEmptyList::single(value.into());
        match key.find(self) {
            Some(idx) => {
                let entry = &mut self.entries[idx];
                let prev = std::mem::replace(&mut entry.value, value);
                Some(prev.take_first())
            }
            None => {
                self.entries.push(HeaderEntry { key, value });
                None
            }
        }
    }

    pub fn append(&mut self, key: HeaderName, value: impl Into<HeaderValue>) -> bool {
        match key.find(self) {
            Some(idx) => {
                let entry = &mut self.entries[idx];
                entry.value.push(value.into());
                true
            }
            None => {
                let value = NonEmptyList::single(value.into());
                self.entries.push(HeaderEntry { key, value });
                false
            }
        }
    }

    pub fn remove(&mut self, key: impl AsHeaderName) -> Option<HeaderValue> {
        match key.find(self) {
            Some(idx) => {
                let entry = self.entries.remove(idx);
                Some(entry.value.take_first())
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
}

impl<'a> Extend<(&'a HeaderName, super::non_empty_list::Iter<'a, HeaderValue>)> for Headers {
    fn extend<
        T: IntoIterator<Item = (&'a HeaderName, super::non_empty_list::Iter<'a, HeaderValue>)>,
    >(
        &mut self,
        iter: T,
    ) {
        for (name, values) in iter {
            for val in values {
                self.append(name.clone(), val.clone());
            }
        }
    }
}

impl Extend<(HeaderName, super::non_empty_list::IntoIter<HeaderValue>)> for Headers {
    fn extend<
        T: IntoIterator<Item = (HeaderName, super::non_empty_list::IntoIter<HeaderValue>)>,
    >(
        &mut self,
        iter: T,
    ) {
        for (name, values) in iter {
            for val in values {
                self.append(name.clone(), val.clone());
            }
        }
    }
}

impl Extend<(HeaderName, HeaderValue)> for Headers {
    fn extend<T: IntoIterator<Item = (HeaderName, HeaderValue)>>(&mut self, iter: T) {
        for (name, value) in iter {
            self.append(name, value);
        }
    }
}

impl<'a> Extend<(&'a HeaderName, &'a HeaderValue)> for Headers {
    fn extend<T: IntoIterator<Item = (&'a HeaderName, &'a HeaderValue)>>(&mut self, iter: T) {
        for (name, value) in iter {
            self.append(name.clone(), value.clone());
        }
    }
}

pub struct GetAll<'a> {
    iter: Option<super::non_empty_list::Iter<'a, HeaderValue>>,
}

impl<'a> Iterator for GetAll<'a> {
    type Item = &'a HeaderValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter {
            None => None,
            Some(ref mut x) => x.next(),
        }
    }
}

pub struct Keys<'a> {
    iter: std::slice::Iter<'a, HeaderEntry>,
}

impl<'a> Iterator for Keys<'a> {
    type Item = &'a HeaderName;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| &x.key)
    }
}

pub struct Iter<'a> {
    entries: &'a Vec<HeaderEntry>,
    index: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a HeaderName, super::non_empty_list::Iter<'a, HeaderValue>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.entries.len() {
            return None;
        }

        let entry = self.entries.get(self.index)?;
        self.index += 1;
        Some((&entry.key, entry.value.iter()))
    }
}

impl<'a> IntoIterator for &'a Headers {
    type Item = (&'a HeaderName, super::non_empty_list::Iter<'a, HeaderValue>);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IntoIter {
    entries: Vec<HeaderEntry>,
}

impl Iterator for IntoIter {
    type Item = (HeaderName, super::non_empty_list::IntoIter<HeaderValue>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.entries.is_empty() {
            return None;
        }

        let entry = self.entries.remove(0);
        let key = entry.key.clone();
        Some((key, entry.value.into_iter()))
    }
}

impl IntoIterator for Headers {
    type Item = (HeaderName, super::non_empty_list::IntoIter<HeaderValue>);
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            entries: self.entries,
        }
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
    use crate::headers::value::HeaderValue;

    use super::Headers;

    #[test]
    fn should_insert() {
        let mut headers = Headers::new();
        headers.insert(
            "Accept".try_into().unwrap(),
            HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "Content Length".try_into().unwrap(),
            HeaderValue::from_static("120"),
        );

        assert_eq!(headers.len(), 2)
    }

    #[test]
    fn should_be_case_insensitive() {
        let mut headers = Headers::new();
        headers.insert("abc".try_into().unwrap(), HeaderValue::from_static("1"));
        headers.insert("xyz".try_into().unwrap(), HeaderValue::from_static("2"));
        headers.insert("JKL".try_into().unwrap(), HeaderValue::from_static("3"));

        assert_eq!(headers.get("ABC"), Some(&HeaderValue::from_static("1")));
        assert_eq!(headers.get("XyZ"), Some(&HeaderValue::from_static("2")));
        assert_eq!(headers.get("jKl"), Some(&HeaderValue::from_static("3")));
    }

    #[test]
    fn should_get_mut() {
        let mut headers = Headers::new();
        headers.insert(
            "Accept".try_into().unwrap(),
            HeaderValue::from_static("hello"),
        );
        *headers.get_mut("accept").unwrap() = HeaderValue::from_static("hello-world");

        assert_eq!(
            headers.get("accept"),
            Some(&HeaderValue::from_static("hello-world"))
        );
    }

    #[test]
    fn should_get_all() {
        let mut headers = Headers::new();
        headers.append(
            "Fruits".try_into().unwrap(),
            HeaderValue::from_static("apple"),
        );
        headers.append(
            "Fruits".try_into().unwrap(),
            HeaderValue::from_static("strawberry"),
        );
        headers.append(
            "Fruits".try_into().unwrap(),
            HeaderValue::from_static("banana"),
        );

        assert_eq!(
            headers.get("fruits"),
            Some(&HeaderValue::from_static("apple"))
        );
        assert_eq!(
            headers
                .get_all("fruits")
                .cloned()
                .collect::<Vec<HeaderValue>>(),
            vec![
                HeaderValue::from_static("apple"),
                HeaderValue::from_static("strawberry"),
                HeaderValue::from_static("banana")
            ]
        );
    }

    #[test]
    fn should_replace_existing_on_insert() {
        let mut headers = Headers::new();
        headers.insert(
            "Accept".try_into().unwrap(),
            HeaderValue::from_static("text/html"),
        );
        headers.insert(
            "Accept-Encoding".try_into().unwrap(),
            HeaderValue::from_static("gzip"),
        );

        headers.insert(
            "accept".try_into().unwrap(),
            HeaderValue::from_static("text/plain"),
        );
        headers.insert(
            "accept-encoding".try_into().unwrap(),
            HeaderValue::from_static("br"),
        );

        assert_eq!(
            headers.get("accept"),
            Some(&HeaderValue::from_static("text/plain"))
        );
        assert_eq!(
            headers.get("accept-encoding"),
            Some(&HeaderValue::from_static("br"))
        )
    }

    #[test]
    fn should_remove() {
        let mut headers = Headers::new();
        headers.insert("abc".try_into().unwrap(), HeaderValue::from_static("1"));
        headers.insert("xyz".try_into().unwrap(), HeaderValue::from_static("2"));
        headers.insert("JKL".try_into().unwrap(), HeaderValue::from_static("3"));

        assert_eq!(headers.remove("abc"), Some(HeaderValue::from_static("1")));
        assert_eq!(headers.remove("abc"), None);

        assert_eq!(headers.remove("xYz"), Some(HeaderValue::from_static("2")));
        assert_eq!(headers.remove("xYz"), None);

        assert_eq!(headers.remove("jKL"), Some(HeaderValue::from_static("3")));
        assert_eq!(headers.remove("jKL"), None);
    }

    #[test]
    fn should_iter_over_all_entries() {
        let mut headers = Headers::new();
        headers.append("numbers".try_into().unwrap(), HeaderValue::from_static("1"));
        headers.append("numbers".try_into().unwrap(), HeaderValue::from_static("2"));
        headers.append("numbers".try_into().unwrap(), HeaderValue::from_static("3"));
        headers.append(
            "fruits".try_into().unwrap(),
            HeaderValue::from_static("apple"),
        );
        headers.append(
            "fruits".try_into().unwrap(),
            HeaderValue::from_static("strawberry"),
        );
        headers.append(
            "food".try_into().unwrap(),
            HeaderValue::from_static("pizza"),
        );

        let mut iter = headers.iter();

        let (numbers_name, numbers) = iter.next().unwrap();
        assert_eq!(numbers_name.as_str(), "numbers");
        assert_eq!(
            numbers.cloned().collect::<Vec<HeaderValue>>(),
            vec![
                HeaderValue::from_static("1"),
                HeaderValue::from_static("2"),
                HeaderValue::from_static("3")
            ]
        );

        let (fruits_name, fruits) = iter.next().unwrap();
        assert_eq!(fruits_name.as_str(), "fruits");
        assert_eq!(
            fruits.cloned().collect::<Vec<HeaderValue>>(),
            vec![
                HeaderValue::from_static("apple"),
                HeaderValue::from_static("strawberry")
            ]
        );

        let (food_name, foods) = iter.next().unwrap();
        assert_eq!(food_name.as_str(), "food");
        assert_eq!(
            foods.cloned().collect::<Vec<HeaderValue>>(),
            vec![HeaderValue::from_static("pizza")]
        );

        assert!(iter.next().is_none())
    }

    #[test]
    fn should_into_iter_over_all_entries() {
        let mut headers = Headers::new();
        headers.append("numbers".try_into().unwrap(), HeaderValue::from_static("1"));
        headers.append("numbers".try_into().unwrap(), HeaderValue::from_static("2"));
        headers.append("numbers".try_into().unwrap(), HeaderValue::from_static("3"));
        headers.append(
            "fruits".try_into().unwrap(),
            HeaderValue::from_static("apple"),
        );
        headers.append(
            "fruits".try_into().unwrap(),
            HeaderValue::from_static("strawberry"),
        );
        headers.append(
            "food".try_into().unwrap(),
            HeaderValue::from_static("pizza"),
        );

        let mut iter = headers.into_iter();

        let (numbers_name, numbers) = iter.next().unwrap();
        assert_eq!(numbers_name.as_str(), "numbers");
        assert_eq!(
            numbers.collect::<Vec<HeaderValue>>(),
            vec![
                HeaderValue::from_static("1"),
                HeaderValue::from_static("2"),
                HeaderValue::from_static("3")
            ]
        );

        let (fruits_name, fruits) = iter.next().unwrap();
        assert_eq!(fruits_name.as_str(), "fruits");
        assert_eq!(
            fruits.collect::<Vec<HeaderValue>>(),
            vec![
                HeaderValue::from_static("apple"),
                HeaderValue::from_static("strawberry")
            ]
        );

        let (food_name, foods) = iter.next().unwrap();
        assert_eq!(food_name.as_str(), "food");
        assert_eq!(
            foods.collect::<Vec<HeaderValue>>(),
            vec![HeaderValue::from_static("pizza")]
        );

        assert!(iter.next().is_none());
    }

    #[test]
    fn should_extend_headers() {
        let mut first = Headers::new();
        first.append("numbers".try_into().unwrap(), HeaderValue::from_static("1"));
        first.append(
            "fruits".try_into().unwrap(),
            HeaderValue::from_static("apple"),
        );

        let mut second = Headers::new();
        second.append("color".try_into().unwrap(), HeaderValue::from_static("red"));
        second.append(
            "animal".try_into().unwrap(),
            HeaderValue::from_static("fox"),
        );

        first.extend(second);

        assert_eq!(first.get("numbers"), Some(&HeaderValue::from_static("1")));
        assert_eq!(
            first.get("fruits"),
            Some(&HeaderValue::from_static("apple"))
        );
        assert_eq!(first.get("color"), Some(&HeaderValue::from_static("red")));
        assert_eq!(first.get("animal"), Some(&HeaderValue::from_static("fox")));
    }
}
