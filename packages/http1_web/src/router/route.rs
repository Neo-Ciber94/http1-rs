use std::{borrow::Cow, str::Split};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RouteSegment<'a> {
    // A static route segment: /static
    Static(Cow<'a, str>),

    // A dynamic segment: /:param
    Dynamic(Cow<'a, str>),

    // The rest of the path: /:rest*
    CatchAll(Cow<'a, str>),
}

impl Ord for RouteSegment<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            // Static route have priority over all other
            (RouteSegment::Static(a), RouteSegment::Static(b)) => a.cmp(b),
            (RouteSegment::Static(_), _) => std::cmp::Ordering::Greater,
            (_, RouteSegment::Static(_)) => std::cmp::Ordering::Less,

            // Then dynamic
            (RouteSegment::Dynamic(a), RouteSegment::Dynamic(b)) => a.cmp(b),
            (RouteSegment::Dynamic(_), RouteSegment::CatchAll(_)) => std::cmp::Ordering::Greater,

            // And lastly catch-all
            (RouteSegment::CatchAll(_), RouteSegment::Dynamic(_)) => std::cmp::Ordering::Less,
            (RouteSegment::CatchAll(a), RouteSegment::CatchAll(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for RouteSegment<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Route<'a>(Vec<RouteSegment<'a>>);

impl Route<'_> {
    pub fn iter(&self) -> std::slice::Iter<'_, RouteSegment<'_>> {
        self.0.iter()
    }
}

impl<'a> From<&'a str> for Route<'a> {
    fn from(value: &'a str) -> Self {
        let segments = route_segments(value).collect::<Vec<_>>();
        Route(segments)
    }
}

#[derive(Clone)]
pub struct RouteSegmentsIter<'a>(Split<'a, &'a str>);

impl<'a> Iterator for RouteSegmentsIter<'a> {
    type Item = RouteSegment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|part| match part {
            _ if part.starts_with(":") => {
                if part.ends_with("*") {
                    let len = part.len();
                    RouteSegment::CatchAll(Cow::Borrowed(&part[1..(len - 1)]))
                } else {
                    RouteSegment::Dynamic(Cow::Borrowed(&part[1..]))
                }
            }
            _ => RouteSegment::Static(Cow::Borrowed(part)),
        })
    }
}

impl Eq for RouteSegmentsIter<'_> {}

impl PartialEq for RouteSegmentsIter<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.clone().eq(other.0.clone())
    }
}

impl Ord for RouteSegmentsIter<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let mut self_iter = self.clone();
        let mut other_iter = other.clone();

        loop {
            match (self_iter.next(), other_iter.next()) {
                (Some(s_segment), Some(o_segment)) => {
                    // Compare segments and return if they differ
                    let cmp = s_segment.cmp(&o_segment);
                    if cmp != std::cmp::Ordering::Equal {
                        return cmp;
                    }
                }
                (None, Some(_)) => return std::cmp::Ordering::Less, // self is shorter
                (Some(_), None) => return std::cmp::Ordering::Greater, // other is shorter
                (None, None) => return std::cmp::Ordering::Equal, // both are equal in length and content
            }
        }
    }
}

impl PartialOrd for RouteSegmentsIter<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub fn get_segments(mut route: &str) -> std::str::Split<'_, &str> {
    if route.starts_with("/") {
        route = &route[1..];
    }

    if route.ends_with("/") {
        route = &route[..(route.len() - 1)];
    }

    route.split("/")
}

fn route_segments(route: &str) -> RouteSegmentsIter {
    RouteSegmentsIter(get_segments(route))
}
