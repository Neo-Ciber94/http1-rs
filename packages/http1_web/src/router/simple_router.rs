use std::collections::{BTreeMap, HashMap};

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
        assert!(!route.starts_with("/"), "route should start with '/'");
        self.routes.insert(Route::from(route), value);
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
    let route_segments = route.iter();

    for (index, part) in route_segments.enumerate() {
        let (_, segment) = get_segments(path)
            .enumerate()
            .find(|(pos, _)| *pos == index)?;

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

    Some(route)
}
