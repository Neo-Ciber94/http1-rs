use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign},
};

use http1::method::Method;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MethodRoute(u16);

impl MethodRoute {
    pub const GET: MethodRoute = MethodRoute(0b000000001);
    pub const POST: MethodRoute = MethodRoute(0b000000010);
    pub const PUT: MethodRoute = MethodRoute(0b000000100);
    pub const DELETE: MethodRoute = MethodRoute(0b000001000);
    pub const PATCH: MethodRoute = MethodRoute(0b000010000);
    pub const OPTIONS: MethodRoute = MethodRoute(0b000100000);
    pub const HEAD: MethodRoute = MethodRoute(0b001000000);
    pub const TRACE: MethodRoute = MethodRoute(0b010000000);
    pub const CONNECT: MethodRoute = MethodRoute(0b100000000);

    pub fn from_method(method: &Method) -> Self {
        MethodRoute::try_from(method).expect("invalid method")
    }

    pub const fn any() -> Self {
        MethodRoute(u16::MAX)
    }

    pub const fn contains(&self, other: MethodRoute) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn into_methods(self) -> impl Iterator<Item = Method> {
        MethodRoute::all().into_iter().filter_map(move |m| {
            if self.contains(m) {
                m.as_method().ok()
            } else {
                None
            }
        })
    }

    pub fn as_method(self) -> Result<Method, MethodRoute> {
        match self {
            MethodRoute::GET => Ok(Method::GET),
            MethodRoute::POST => Ok(Method::POST),
            MethodRoute::PUT => Ok(Method::PUT),
            MethodRoute::DELETE => Ok(Method::DELETE),
            MethodRoute::PATCH => Ok(Method::PATCH),
            MethodRoute::OPTIONS => Ok(Method::OPTIONS),
            MethodRoute::HEAD => Ok(Method::HEAD),
            MethodRoute::TRACE => Ok(Method::TRACE),
            MethodRoute::CONNECT => Ok(Method::CONNECT),
            _ => Err(self),
        }
    }

    pub const fn all() -> [MethodRoute; 9] {
        [
            MethodRoute::GET,
            MethodRoute::POST,
            MethodRoute::PUT,
            MethodRoute::DELETE,
            MethodRoute::PATCH,
            MethodRoute::OPTIONS,
            MethodRoute::HEAD,
            MethodRoute::TRACE,
            MethodRoute::CONNECT,
        ]
    }
}

#[derive(Debug)]
pub struct InvalidMethodRoute(String);

impl std::error::Error for InvalidMethodRoute {}

impl Display for InvalidMethodRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid method route: {}", self.0)
    }
}

impl<'a> TryFrom<&'a Method> for MethodRoute {
    type Error = InvalidMethodRoute;

    fn try_from(value: &'a Method) -> Result<Self, Self::Error> {
        match value {
            Method::GET => Ok(MethodRoute::GET),
            Method::POST => Ok(MethodRoute::POST),
            Method::PUT => Ok(MethodRoute::PUT),
            Method::DELETE => Ok(MethodRoute::DELETE),
            Method::PATCH => Ok(MethodRoute::PATCH),
            Method::OPTIONS => Ok(MethodRoute::OPTIONS),
            Method::HEAD => Ok(MethodRoute::HEAD),
            Method::CONNECT => Ok(MethodRoute::CONNECT),
            Method::TRACE => Ok(MethodRoute::TRACE),
            Method::ExtensionMethod(s) => Err(InvalidMethodRoute(s.clone())),
        }
    }
}

impl Display for MethodRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let all_methods = MethodRoute::all();
        let methods = all_methods.iter().filter(|m| self.contains(**m));
        let count = methods.clone().count();

        for (idx, method) in methods.enumerate() {
            if self.contains(*method) {
                write_method(f, *method)?;

                if idx < count - 1 {
                    write!(f, " ")?;
                }
            }
        }

        Ok(())
    }
}

fn write_method(f: &mut std::fmt::Formatter<'_>, method: MethodRoute) -> std::fmt::Result {
    match method {
        _ if MethodRoute::GET == method => write!(f, "GET"),
        _ if MethodRoute::POST == method => write!(f, "POST"),
        _ if MethodRoute::PUT == method => write!(f, "PUT"),
        _ if MethodRoute::DELETE == method => write!(f, "DELETE"),
        _ if MethodRoute::PATCH == method => write!(f, "PATCH"),
        _ if MethodRoute::OPTIONS == method => write!(f, "OPTIONS"),
        _ if MethodRoute::HEAD == method => write!(f, "HEAD"),
        _ if MethodRoute::TRACE == method => write!(f, "TRACE"),
        _ if MethodRoute::CONNECT == method => write!(f, "CONNECT"),
        _ => panic!("Unable to determine method: {method:?}"),
    }
}

impl BitAnd for MethodRoute {
    type Output = MethodRoute;

    fn bitand(self, rhs: Self) -> Self::Output {
        MethodRoute(self.0 & rhs.0)
    }
}

impl BitAndAssign for MethodRoute {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl BitOr for MethodRoute {
    type Output = MethodRoute;

    fn bitor(self, rhs: Self) -> Self::Output {
        MethodRoute(self.0 | rhs.0)
    }
}

impl BitOrAssign for MethodRoute {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http1::method::Method;

    #[test]
    fn should_create_method_route_for_all_methods() {
        assert_eq!(MethodRoute::from_method(&Method::GET), MethodRoute::GET);
        assert_eq!(MethodRoute::from_method(&Method::POST), MethodRoute::POST);
        assert_eq!(MethodRoute::from_method(&Method::PUT), MethodRoute::PUT);
        assert_eq!(
            MethodRoute::from_method(&Method::DELETE),
            MethodRoute::DELETE
        );
        assert_eq!(MethodRoute::from_method(&Method::PATCH), MethodRoute::PATCH);
        assert_eq!(
            MethodRoute::from_method(&Method::OPTIONS),
            MethodRoute::OPTIONS
        );
        assert_eq!(MethodRoute::from_method(&Method::HEAD), MethodRoute::HEAD);
        assert_eq!(
            MethodRoute::from_method(&Method::CONNECT),
            MethodRoute::CONNECT
        );
        assert_eq!(MethodRoute::from_method(&Method::TRACE), MethodRoute::TRACE);
    }

    #[test]
    fn should_handle_bitwise_or_for_methods() {
        let combined_route = MethodRoute::GET | MethodRoute::POST;
        assert!(combined_route.contains(MethodRoute::GET));
        assert!(combined_route.contains(MethodRoute::POST));
        assert!(!combined_route.contains(MethodRoute::PUT));
    }

    #[test]
    fn should_handle_bitwise_and_for_methods() {
        let combined_route = MethodRoute::GET | MethodRoute::POST;
        let intersection = combined_route & MethodRoute::GET;
        assert_eq!(intersection, MethodRoute::GET);
    }

    #[test]
    fn should_check_if_combined_route_contains_method() {
        let route = MethodRoute::GET | MethodRoute::POST | MethodRoute::PUT;
        assert!(route.contains(MethodRoute::GET));
        assert!(route.contains(MethodRoute::POST));
        assert!(route.contains(MethodRoute::PUT));
        assert!(!route.contains(MethodRoute::DELETE));
    }

    #[test]
    fn should_return_max_route_for_any() {
        let any_route = MethodRoute::any();
        assert!(any_route.contains(MethodRoute::GET));
        assert!(any_route.contains(MethodRoute::POST));
        assert!(any_route.contains(MethodRoute::PUT));
        assert!(any_route.contains(MethodRoute::DELETE));
    }

    #[test]
    fn should_return_error_for_invalid_method() {
        let invalid_method = Method::ExtensionMethod("INVALID".to_string());
        let result = MethodRoute::try_from(&invalid_method);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "invalid method route: INVALID");
        }
    }

    #[test]
    fn should_support_bitwise_or_assignment() {
        let mut route = MethodRoute::GET;
        route |= MethodRoute::POST;
        assert!(route.contains(MethodRoute::GET));
        assert!(route.contains(MethodRoute::POST));
    }

    #[test]
    fn should_support_bitwise_and_assignment() {
        let mut route = MethodRoute::GET | MethodRoute::POST;
        route &= MethodRoute::GET;
        assert!(route.contains(MethodRoute::GET));
        assert!(!route.contains(MethodRoute::POST));
    }

    #[test]
    fn should_return_correct_method_for_as_method() {
        assert_eq!(MethodRoute::GET.as_method(), Ok(Method::GET));
        assert_eq!(MethodRoute::POST.as_method(), Ok(Method::POST));
        assert_eq!(MethodRoute::PUT.as_method(), Ok(Method::PUT));
        assert_eq!(MethodRoute::DELETE.as_method(), Ok(Method::DELETE));
        assert_eq!(MethodRoute::PATCH.as_method(), Ok(Method::PATCH));
        assert_eq!(MethodRoute::OPTIONS.as_method(), Ok(Method::OPTIONS));
        assert_eq!(MethodRoute::HEAD.as_method(), Ok(Method::HEAD));
        assert_eq!(MethodRoute::TRACE.as_method(), Ok(Method::TRACE));
        assert_eq!(MethodRoute::CONNECT.as_method(), Ok(Method::CONNECT));

        // Invalid cases
        assert!(MethodRoute::any().as_method().is_err());
        assert!((MethodRoute::GET | MethodRoute::POST).as_method().is_err());
    }

    #[test]
    fn should_return_correct_methods_for_into_methods() {
        let route = MethodRoute::GET | MethodRoute::POST | MethodRoute::DELETE;
        let methods: Vec<Method> = route.into_methods().collect();

        assert_eq!(methods, vec![Method::GET, Method::POST, Method::DELETE]);

        // Test with the `any` method
        let all_routes = MethodRoute::any();
        let all_methods: Vec<Method> = all_routes.into_methods().collect();

        assert_eq!(
            all_methods,
            vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
                Method::HEAD,
                Method::TRACE,
                Method::CONNECT,
            ]
        );
    }
}
