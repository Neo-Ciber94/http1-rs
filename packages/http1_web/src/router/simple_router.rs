use std::collections::{BTreeMap, HashMap};

use crate::router::route::RouteSegment;

use super::{
    route::{self, get_segments, Route},
    Match, Params,
};

#[derive(Default, Clone, Debug)]
pub(crate) struct SimpleRouter<'a, T> {
    routes: BTreeMap<Route<'a>, T>,
}

impl<'a, T> SimpleRouter<'a, T> {
    pub fn new() -> Self {
        SimpleRouter {
            routes: Default::default(),
        }
    }

    pub fn insert(&mut self, route: &'a str, value: T) {
        assert!(route.starts_with("/"), "route should start with '/'");
        let r = Route::from(route);

        // If there is a catch-all it should be the last param
        let segments = r.iter();
        let len = segments.len();

        for (index, segment) in segments.enumerate() {
            if matches!(segment, RouteSegment::CatchAll(_)) && index < len - 1 {
                panic!("catch-all segment must be the last route segment: {route:?}");
            }
        }

        self.routes.insert(r, value);
    }

    pub fn find(&'a self, path: &'a str) -> Option<Match<'a, T>> {
        let mut params_map = HashMap::new();

        for (route, value) in self.routes.iter() {
            match find_route(route, path, &mut params_map) {
                Some(_) => {
                    let params = Params(std::mem::take(&mut params_map));
                    return Some(Match { params, value });
                }
                None => {
                    params_map.clear();
                }
            }
        }

        None
    }
}

fn find_route<'a>(
    route: &'a Route,
    path: &str,
    params_map: &mut HashMap<String, String>,
) -> Option<&'a Route<'a>> {
    let mut segments = get_segments(path).enumerate().peekable();
    let route_segments = route.iter();

    for (index, part) in route_segments.enumerate() {
        let (_, segment) = segments.next()?;

        match part {
            route::RouteSegment::Static(param) => {
                if param != segment {
                    return None;
                }
            }
            route::RouteSegment::Dynamic(param_name) => {
                params_map.insert(param_name.to_string(), segment.to_owned());
            }
            route::RouteSegment::CatchAll(param_name) => {
                let rest = get_segments(path).skip(index).collect::<Vec<_>>().join("/");
                params_map.insert(param_name.to_string(), rest);
                return Some(route);
            }
        }
    }

    // We still have elements
    if segments.peek().is_some() {
        return None;
    }

    Some(route)
}

#[cfg(test)]
mod tests {
    use super::SimpleRouter;

    #[test]
    #[should_panic]
    fn should_fail_to_add_missing_splash() {
        SimpleRouter::new().insert("my_route", ());
    }

    #[test]
    #[should_panic]
    fn should_fail_to_add_catch_all() {
        SimpleRouter::new().insert("/other/:path*/third", ());
    }

    #[test]
    fn ignore_trailing_slash() {
        let mut router = SimpleRouter::new();
        router.insert("/hello/", 1);

        assert!(router.find("/hello").is_some());
        assert!(router.find("/hello/").is_some());
    }

    #[test]
    fn should_find_static_route() {
        let mut router = SimpleRouter::new();
        router.insert("/", 1);
        router.insert("/first", 2);
        router.insert("/first/second", 3);

        assert_eq!(router.find("/").unwrap().value, &1);
        assert_eq!(router.find("/first").unwrap().value, &2);
        assert_eq!(router.find("/first/second").unwrap().value, &3);
        assert_eq!(router.find("/third"), None);
        assert_eq!(router.find("/first/third"), None);
        assert_eq!(router.find("/first/second/third"), None);
    }

    #[test]
    fn should_find_dynamic_route() {
        let mut router = SimpleRouter::new();
        router.insert("/fruits/:name", 1);
        router.insert("/fruits/:name/color", 2);
        router.insert("/:id", 3);

        let match1 = router.find("/fruits/apple").unwrap();
        assert_eq!(match1.value, &1);
        assert_eq!(match1.params.get("name"), Some("apple"));

        let match2 = router.find("/fruits/orange/color").unwrap();
        assert_eq!(match2.value, &2);
        assert_eq!(match2.params.get("name"), Some("orange"));

        let match3 = router.find("/color").unwrap();
        assert_eq!(match3.value, &3);
    }

    #[test]
    fn should_find_catch_all_route() {
        let mut router = SimpleRouter::new();
        router.insert("/languages/:rest*", 1);
        router.insert("/languages/english/:other*", 2);
        router.insert("/:params*", 3);
        router.insert("/some_path*", 4);

        let match1 = router.find("/languages/unknown/missing").unwrap();
        assert_eq!(match1.value, &1);
        assert_eq!(match1.params.get("rest"), Some("unknown/missing"));

        let match2 = router.find("/languages/english/cities").unwrap();
        assert_eq!(match2.value, &2);
        assert_eq!(match2.params.get("other"), Some("cities"));

        let match3 = router.find("/books").unwrap();
        assert_eq!(match3.value, &3);

        let match4 = router.find("/some_path*").unwrap();
        assert_eq!(match4.value, &4);
    }
}
