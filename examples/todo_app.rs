use http1::server::Server;
use http1_web::{app::App, into_response::IntoResponse};

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let addr = "127.0.0.1:5000".parse().unwrap();

    Server::new(addr)
        .on_ready(|addr| log::debug!("Listening on http://{addr}"))
        .start(App::new().get("/", hello))
}

fn hello() -> impl IntoResponse {
    "Hello World!"
}

mod db {
    use std::{
        any::Any,
        collections::HashMap,
        sync::{atomic::AtomicU64, Arc, RwLock},
    };

    #[derive(Debug, PartialEq, Eq, Hash)]
    pub enum Table {
        User,
        Todo,
        Session,
    }

    struct Id(AtomicU64);
    impl Id {
        const fn new() -> Self {
            Id(AtomicU64::new(0))
        }

        fn next(&self) -> u64 {
            self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
        }
    }

    #[derive(Debug, Clone)]
    pub struct User {
        pub id: u64,
        pub username: String,
    }

    #[derive(Debug, Clone)]
    pub struct Todo {
        pub id: u64,
        pub title: String,
        pub description: Option<String>,
        pub is_done: bool,
        pub user_id: u64,
    }

    #[derive(Debug, Clone)]
    pub struct Session {
        pub id: String,
        pub user_id: u64,
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    enum Key {
        Number(u64),
        String(String),
    }

    #[derive(Clone)]
    pub struct DB(Arc<RwLock<HashMap<Table, HashMap<Key, Box<dyn Any + Send + Sync>>>>>);

    impl DB {
        pub fn new() -> Self {
            let tables = HashMap::from_iter([
                (Table::User, Default::default()),
                (Table::Todo, Default::default()),
                (Table::Session, Default::default()),
            ]);

            DB(Arc::new(RwLock::new(tables)))
        }
    }

    static NEXT_USER_ID: Id = Id::new();
    static NEXT_TODO_ID: Id = Id::new();

    #[derive(Debug)]
    pub enum DBError {
        FailedToRead,
        FailedToWrite,
    }

    pub fn insert_user(db: &DB, username: String) -> Result<User, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::User)
            .expect("user table should exists");

        let id = NEXT_USER_ID.next();
        let user = User { id, username };
        records.insert(Key::Number(id), Box::new(user.clone()));
        Ok(user)
    }

    pub fn update_user(db: &DB, user: User) -> Result<Option<User>, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::User)
            .expect("user table should exists");

        match records
            .get_mut(&Key::Number(user.id))
            .and_then(|x| x.downcast_mut::<User>())
        {
            Some(user_to_update) => {
                user_to_update.username = user.username;
                Ok(Some(user_to_update.clone()))
            }
            None => Ok(None),
        }
    }

    pub fn delete_user(db: &DB, id: u64) -> Result<Option<User>, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::User)
            .expect("user table should exists");

        let deleted = records
            .remove(&Key::Number(id))
            .and_then(|x| x.downcast::<User>().ok())
            .map(|x| *x);

        Ok(deleted)
    }

    pub fn get_user(db: &DB, id: u64) -> Result<Option<User>, DBError> {
        let lock = db.0.read().map_err(|_| DBError::FailedToRead)?;
        let records = lock.get(&Table::User).expect("user table should exists");

        records
            .get(&Key::Number(id))
            .and_then(|x| x.downcast_ref::<User>())
            .cloned()
            .map(Ok)
            .transpose()
    }

    pub fn get_all_user(db: &DB) -> Result<Vec<User>, DBError> {
        let lock = db.0.read().map_err(|_| DBError::FailedToRead)?;
        let records = lock.get(&Table::User).expect("user table should exists");

        let users = records
            .values()
            .filter_map(|x| x.downcast_ref::<User>())
            .cloned()
            .collect::<Vec<_>>();

        Ok(users)
    }

    pub fn insert_todo(
        db: &DB,
        title: String,
        description: Option<String>,
        user_id: u64,
    ) -> Result<Todo, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::Todo)
            .expect("todos table should exists");

        let id = NEXT_TODO_ID.next();
        let todo: Todo = Todo {
            id,
            title,
            description,
            is_done: false,
            user_id,
        };
        records.insert(Key::Number(id), Box::new(todo.clone()));
        Ok(todo)
    }

    pub fn update_todo(db: &DB, todo: Todo) -> Result<Option<Todo>, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::Todo)
            .expect("todo table should exists");

        match records
            .get_mut(&Key::Number(todo.id))
            .and_then(|x| x.downcast_mut::<Todo>())
        {
            Some(to_update) => {
                *to_update = todo;
                Ok(Some(to_update.clone()))
            }
            None => Ok(None),
        }
    }

    pub fn delete_todo(db: &DB, id: u64) -> Result<Option<Todo>, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::Todo)
            .expect("todos table should exists");

        let deleted = records
            .remove(&Key::Number(id))
            .and_then(|x| x.downcast::<Todo>().ok())
            .map(|x| *x);

        Ok(deleted)
    }

    pub fn get_todo(db: &DB, id: u64) -> Result<Option<Todo>, DBError> {
        let lock = db.0.read().map_err(|_| DBError::FailedToRead)?;
        let todos = lock.get(&Table::Todo).expect("todos table should exists");

        todos
            .get(&Key::Number(id))
            .and_then(|x| x.downcast_ref::<Todo>())
            .cloned()
            .map(Ok)
            .transpose()
    }

    pub fn get_all_todos(db: &DB) -> Result<Vec<Todo>, DBError> {
        let lock = db.0.read().map_err(|_| DBError::FailedToRead)?;
        let todos = lock.get(&Table::Todo).expect("todos table should exists");

        let todos = todos
            .values()
            .filter_map(|x| x.downcast_ref::<Todo>())
            .cloned()
            .collect::<Vec<_>>();

        Ok(todos)
    }

    pub fn create_session(db: &DB, user_id: u64) -> Result<Session, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::Session)
            .expect("sessions table should exists");

        let id = http1::rng::sequence::<http1::rng::random::Alphanumeric>()
            .take(32)
            .collect::<String>();
        let session = Session {
            id: id.clone(),
            user_id,
        };
        records.insert(Key::String(id), Box::new(session.clone()));
        Ok(session)
    }

    pub fn get_session_by_id(db: &DB, session_id: String) -> Result<Option<Session>, DBError> {
        let lock = db.0.read().map_err(|_| DBError::FailedToRead)?;
        let sessions = lock
            .get(&Table::Session)
            .expect("sessions table should exists");

        sessions
            .get(&Key::String(session_id))
            .and_then(|x| x.downcast_ref::<Session>())
            .cloned()
            .map(Ok)
            .transpose()
    }

    pub fn get_session_user(db: &DB, session_id: String) -> Result<Option<User>, DBError> {
        let session = match get_session_by_id(db, session_id)? {
            Some(s) => s,
            None => return Ok(None),
        };

        get_user(db, session.user_id)
    }

    pub fn remove_session(db: &DB, session_id: String) -> Result<(), DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::Session)
            .expect("sessions table should exists");

        records.remove(&Key::String(session_id));
        Ok(())
    }
}
