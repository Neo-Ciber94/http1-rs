use std::{
    fmt::{Debug, Display},
    str::Split,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RouteSegment {
    // A static route segment: /static
    Static(String),

    // A dynamic segment: /:param
    Dynamic(String),

    // The rest of the path: /:rest*
    CatchAll(String),
}

impl RouteSegment {
    pub fn is_static(&self) -> bool {
        matches!(self, RouteSegment::Static(_))
    }

    pub fn is_dynamic(&self) -> bool {
        matches!(self, RouteSegment::Dynamic(_))
    }

    pub fn is_catch_all(&self) -> bool {
        matches!(self, RouteSegment::CatchAll(_))
    }
}

impl Ord for RouteSegment {
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

impl PartialOrd for RouteSegment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for RouteSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteSegment::Static(s) => write!(f, "/{s}"),
            RouteSegment::Dynamic(s) => write!(f, "/:{s}"),
            RouteSegment::CatchAll(s) => {
                if s.is_empty() {
                    write!(f, "/*")
                } else {
                    write!(f, "/:{s}*")
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Route(Vec<RouteSegment>);

impl Route {
    /// Returns an iterator over the route segments.
    pub fn iter(&self) -> std::slice::Iter<'_, RouteSegment> {
        self.0.iter()
    }

    /// Returns `true` if all the route segments are static.
    pub fn is_static(&self) -> bool {
        self.iter().all(|s| matches!(s, RouteSegment::Static(_)))
    }
}

impl<'a> From<&'a str> for Route {
    fn from(value: &'a str) -> Self {
        let segments = route_segments(value).collect::<Vec<_>>();
        Route(segments)
    }
}

impl From<String> for Route {
    fn from(value: String) -> Self {
        let segments = route_segments(&value).collect::<Vec<_>>();
        Route(segments)
    }
}

impl Debug for Route {
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

impl Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

#[derive(Clone)]
pub struct RouteSegmentsIter<'a>(Split<'a, &'a str>);

impl<'a> Iterator for RouteSegmentsIter<'a> {
    type Item = RouteSegment;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|part| match part {
            _ if part == "*" => RouteSegment::CatchAll(String::new()),
            _ if part.starts_with(":") => {
                if part.ends_with("*") {
                    let len = part.len();
                    RouteSegment::CatchAll(part[1..(len - 1)].to_owned())
                } else {
                    RouteSegment::Dynamic(part[1..].to_owned())
                }
            }
            _ => RouteSegment::Static(part.to_owned()),
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

#[doc(hidden)]
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
            Route::from("/*"),
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
                Route::from("/*"),
                Route::from("/:catch_all*"),
            ]
        )
    }
}
