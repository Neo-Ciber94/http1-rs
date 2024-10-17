use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign},
    u16,
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

    pub fn any() -> Self {
        MethodRoute(u16::MAX)
    }

    pub fn contains(&self, other: MethodRoute) -> bool {
        (self.0 & other.0) != 0
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
        assert_eq!(MethodRoute::from_method(&&Method::PUT), MethodRoute::PUT);
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
}
