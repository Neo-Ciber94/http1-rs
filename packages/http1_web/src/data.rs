use std::{fmt::Debug, marker::PhantomData, ops::Deref, sync::Arc};

use http1::{common::any_map::AnyMap, status::StatusCode};

use crate::{from_request::FromRequest, into_response::IntoResponse};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Data<T>(Arc<T>);

impl<T> Data<T>
where
    T: Send + Sync + 'static,
{
    pub fn new(value: T) -> Self {
        Data(Arc::new(value))
    }
}

impl<T> Clone for Data<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> AsRef<T> for Data<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Data<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[doc(hidden)]
pub struct DataNotFound<T: 'static>(PhantomData<T>);
impl<T: 'static> IntoResponse for DataNotFound<T> {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        eprintln!(
            "Failed to retrieve `{}` from request",
            std::any::type_name::<T>()
        );

        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl<T> FromRequest for Data<T>
where
    T: Send + Sync + 'static,
{
    type Rejection = DataNotFound<T>;

    fn from_request(
        req: http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        match req
            .extensions()
            .get::<Arc<DataMap>>()
            .and_then(|m| m.get::<T>())
            .clone()
        {
            Some(x) => Ok(x),
            None => Err(DataNotFound::<T>(PhantomData)),
        }
    }
}

#[derive(Default)]
pub struct DataMap(AnyMap);

impl DataMap {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert<T>(&mut self, value: T)
    where
        T: Send + Sync + 'static,
    {
        self.0.insert(Data::new(value));
    }

    pub fn get<T>(&self) -> Option<Data<T>>
    where
        T: Send + Sync + 'static,
    {
        self.0.get::<Data<T>>().cloned()
    }
}

impl FromRequest for Arc<DataMap> {
    type Rejection = DataNotFound<DataMap>;

    fn from_request(
        req: http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        req.extensions()
            .get::<Arc<DataMap>>()
            .cloned()
            .ok_or(DataNotFound(PhantomData))
    }
}
