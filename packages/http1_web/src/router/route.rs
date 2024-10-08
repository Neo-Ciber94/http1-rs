use std::{borrow::Cow, fmt::Display, str::Split};

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
            (RouteSegment::Static(_), _) => std::cmp::Ordering::Less,
            (_, RouteSegment::Static(_)) => std::cmp::Ordering::Greater,

            // Then dynamic
            (RouteSegment::Dynamic(a), RouteSegment::Dynamic(b)) => a.cmp(b),
            (RouteSegment::Dynamic(_), RouteSegment::CatchAll(_)) => std::cmp::Ordering::Less,
            (RouteSegment::CatchAll(_), RouteSegment::Dynamic(_)) => std::cmp::Ordering::Greater,

            // And lastly catch-all
            (RouteSegment::CatchAll(a), RouteSegment::CatchAll(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for RouteSegment<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for RouteSegment<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteSegment::Static(s) => write!(f, "/{s}"),
            RouteSegment::Dynamic(s) => write!(f, "/{s}"),
            RouteSegment::CatchAll(s) => write!(f, "/{s}"),
        }
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

impl Display for Route<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "/")?;
        } else {
            for segment in self.0.iter() {
                write!(f, "{}", segment)?;
            }
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::Route;

    #[test]
    fn should_sort_routes() {
        let mut routes = vec![
            Route::from("/static"),
            Route::from("/static/:dynamic"),
            Route::from("/:dynamic"),
            Route::from("/static/:dynamic/static"),
            Route::from("/:dynamic/static"),
            Route::from("/:catch_all*"),
            Route::from("/static/:catch_all*"),
            Route::from("/static/:dynamic/:catch_all*"),
            Route::from("/static/:dynamic/:dynamic/:catch_all*"),
        ];

        routes.sort();

        assert_eq!(
            routes,
            vec![
                Route::from("/static"),
                Route::from("/static/:dynamic"),
                Route::from("/static/:dynamic/static"),
                Route::from("/static/:dynamic/:dynamic/:catch_all*"),
                Route::from("/static/:dynamic/:catch_all*"),
                Route::from("/static/:catch_all*"),
                Route::from("/:dynamic"),
                Route::from("/:dynamic/static"),
                Route::from("/:catch_all*"),
            ]
        )
    }
}
