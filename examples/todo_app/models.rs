use crate::db::KeyValueDatabase;
use datetime::DateTime;
use http1::error::BoxError;
use http1_web::impl_serde_struct;
use std::{fmt::Display, time::Duration};

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub username: String,
}

impl_serde_struct!(User => {
     id: u64,
     username: String,
});

#[derive(Debug, Clone)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub is_done: bool,
    pub user_id: u64,
}

impl_serde_struct!(Todo => {
     id: u64,
     title: String,
     description: Option<String>,
     is_done: bool,
     user_id: u64,
});

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: u64,
    pub created_at: DateTime,
}

impl_serde_struct!(Session => {
     id: String,
     user_id: u64,
     created_at: DateTime
});

const SESSION_DURATION: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct ValidationError {
    field: &'static str,
    message: String,
}
impl ValidationError {
    pub fn new(field: &'static str, message: impl Into<String>) -> Self {
        ValidationError {
            field,
            message: message.into(),
        }
    }
}

impl std::error::Error for ValidationError {}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

pub trait Validate: Sized {
    fn validate_in_place(&mut self) -> Result<(), ValidationError>;
    fn validate(mut self) -> Result<Self, ValidationError> {
        self.validate_in_place()?;
        Ok(self)
    }
}

impl Validate for User {
    fn validate_in_place(&mut self) -> Result<(), ValidationError> {
        if self.username.trim().is_empty() {
            return Err(ValidationError::new("username", "cannot be empty"));
        }

        self.username = self.username.trim().into();

        Ok(())
    }
}

impl Validate for Todo {
    fn validate_in_place(&mut self) -> Result<(), ValidationError> {
        if self.title.trim().is_empty() {
            return Err(ValidationError::new("title", "cannot be empty"));
        }

        if let Some(description) = self.description.as_deref().filter(|x| !x.is_empty()) {
            if description.trim().is_empty() {
                return Err(ValidationError::new("description", "cannot be empty"));
            }
        }

        self.title = self.title.trim().into();
        self.description = self.description.as_deref().map(|x| x.trim().into());

        Ok(())
    }
}

pub fn insert_user(db: &KeyValueDatabase, username: String) -> Result<User, BoxError> {
    if get_user_by_username(db, &username)?.is_some() {
        return Err(String::from("user already exists").into());
    }

    let id = db.incr("next_user_id")? + 1;
    let user = User { id, username }.validate()?;
    db.set(format!("user/{id}"), user.clone())?;
    Ok(user)
}

pub fn get_user(db: &KeyValueDatabase, user_id: u64) -> Result<Option<User>, BoxError> {
    let key = &format!("user/{user_id}");
    let user = db.get::<User>(key)?;
    Ok(user)
}

pub fn get_user_by_username(
    db: &KeyValueDatabase,
    username: &str,
) -> Result<Option<User>, BoxError> {
    let user = db
        .scan::<User>("user/")?
        .into_iter()
        .find(|x| x.username == username);
    Ok(user)
}

pub fn insert_todo(
    db: &KeyValueDatabase,
    title: String,
    description: Option<String>,
    user_id: u64,
) -> Result<Todo, BoxError> {
    let id = db.incr("next_todo_id")? + 1;
    let todo = Todo {
        id,
        title,
        description,
        user_id,
        is_done: false,
    }
    .validate()?;
    db.set(format!("todo/{id}"), todo.clone())?;
    Ok(todo)
}

pub fn update_todo(db: &KeyValueDatabase, mut todo: Todo) -> Result<Option<Todo>, BoxError> {
    let key = &format!("todo/{}", todo.id);

    if db.contains(key)? {
        todo.validate_in_place()?;
        db.set(key, todo.clone())?;
        Ok(Some(todo))
    } else {
        Ok(None)
    }
}

pub fn delete_todo(db: &KeyValueDatabase, id: u64) -> Result<Option<Todo>, BoxError> {
    let key = &format!("todo/{id}");

    match db.get(key)? {
        Some(deleted) => {
            db.del(key)?;
            Ok(Some(deleted))
        }
        None => Ok(None),
    }
}

pub fn get_todo(db: &KeyValueDatabase, todo_id: u64) -> Result<Option<Todo>, BoxError> {
    let key = &format!("todo/{todo_id}");
    let user = db.get::<Todo>(key)?;
    Ok(user)
}

pub fn get_all_todos(db: &KeyValueDatabase, user_id: u64) -> Result<Vec<Todo>, BoxError> {
    let todos = db
        .scan::<Todo>("todo/")?
        .into_iter()
        .filter(|x| x.user_id == user_id)
        .collect::<Vec<_>>();

    Ok(todos)
}

pub fn create_session(db: &KeyValueDatabase, user_id: u64) -> Result<Session, BoxError> {
    let id = rng::sequence::<rng::random::Alphanumeric>()
        .take(32)
        .collect::<String>();

    let session = Session {
        id: id.clone(),
        user_id,
        created_at: DateTime::now_utc(),
    };

    db.set(format!("session/{id}"), session.clone())?;
    Ok(session)
}

pub fn get_session_by_id(
    db: &KeyValueDatabase,
    session_id: String,
) -> Result<Option<Session>, BoxError> {
    remove_expired_sessions(db);

    let key = format!("session/{session_id}");
    let session = db.get::<Session>(key)?;
    Ok(session)
}

pub fn get_session_user(
    db: &KeyValueDatabase,
    session_id: String,
) -> Result<Option<User>, BoxError> {
    let session: Session = match get_session_by_id(db, session_id)? {
        Some(s) => s,
        None => return Ok(None),
    };

    get_user(db, session.user_id)
}

pub fn remove_session(db: &KeyValueDatabase, session_id: String) -> Result<(), BoxError> {
    remove_expired_sessions(db);

    let key = format!("session/{session_id}");
    db.del(key)?;
    Ok(())
}

pub fn remove_expired_sessions(db: &KeyValueDatabase) {
    fn try_remove_expired_sessions(db: &KeyValueDatabase) -> Result<(), BoxError> {
        let now = DateTime::now_utc();
        let deleted = db.retain::<Session>("session/", |_, session| {
            now < (session.created_at + SESSION_DURATION)
        })?;

        if deleted > 0 {
            log::debug!("{deleted} expired sessions deleted");
        }

        Ok(())
    }

    if let Err(err) = try_remove_expired_sessions(db) {
        log::error!("Failed to remove expired sessions: {err}");
    }
}
