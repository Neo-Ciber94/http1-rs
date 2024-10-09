use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::Deref,
    sync::Arc,
};

use http1::common::any_map::AnyMap;

use crate::from_request::FromRequest;

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

pub struct DataNotFound<T>(PhantomData<T>);

impl<T> Debug for DataNotFound<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DataNotFound").finish()
    }
}

impl<T: 'static> Display for DataNotFound<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to retrieve `{}` from request",
            std::any::type_name::<T>()
        )
    }
}

impl<T: 'static> std::error::Error for DataNotFound<T> {}

impl<T> FromRequest for Data<T>
where
    T: Send + Sync + 'static,
{
    fn from_request(
        req: http1::request::Request<http1::body::Body>,
    ) -> Result<Self, http1::error::BoxError> {
        match req.extensions().get::<Data<T>>().cloned() {
            Some(x) => Ok(x),
            None => Err(DataNotFound::<T>(PhantomData).into()),
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
    fn from_request(
        req: http1::request::Request<http1::body::Body>,
    ) -> Result<Self, http1::error::BoxError> {
        req.extensions()
            .get::<Arc<DataMap>>()
            .cloned()
            .ok_or_else(|| DataNotFound::<DataMap>(PhantomData).into())
    }
}
