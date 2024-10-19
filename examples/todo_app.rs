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

    #[derive(Clone)]
    pub struct DB(Arc<RwLock<HashMap<Table, HashMap<u64, Box<dyn Any + Send + Sync>>>>>);

    impl DB {
        pub fn new() -> Self {
            let tables = HashMap::from_iter([
                (Table::User, Default::default()),
                (Table::Todo, Default::default()),
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

    pub fn insert_user(db: DB, username: String) -> Result<User, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let users = lock
            .get_mut(&Table::User)
            .expect("user table should exists");

        let id = NEXT_TODO_ID.next();
        let user = User { id, username };
        users.insert(id, Box::new(user.clone()));
        Ok(user)
    }

    pub fn update_user(db: DB, id: u64, username: String) -> Result<Option<User>, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let users = lock
            .get_mut(&Table::User)
            .expect("user table should exists");

        match users.get_mut(&id).and_then(|x| x.downcast_mut::<User>()) {
            Some(user_to_update) => {
                user_to_update.username = username;
                Ok(Some(user_to_update.clone()))
            }
            None => Ok(None),
        }
    }

    pub fn delete_user(db: DB, id: u64) -> Result<Option<User>, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let users = lock
            .get_mut(&Table::User)
            .expect("user table should exists");

        let deleted_user = users
            .remove(&id)
            .and_then(|x| x.downcast::<User>().ok())
            .map(|x| *x);

        Ok(deleted_user)
    }

    pub fn get_user(db: DB, id: u64) -> Result<Option<User>, DBError> {
        let lock = db.0.read().map_err(|_| DBError::FailedToRead)?;
        let users = lock.get(&Table::User).expect("user table should exists");

        users
            .get(&id)
            .and_then(|x| x.downcast_ref::<User>())
            .cloned()
            .map(Ok)
            .transpose()
    }

    pub fn get_all_user(db: DB) -> Result<Vec<User>, DBError> {
        let lock = db.0.read().map_err(|_| DBError::FailedToRead)?;
        let users = lock.get(&Table::User).expect("user table should exists");

        let users = users
            .values()
            .filter_map(|x| x.downcast_ref::<User>())
            .cloned()
            .collect::<Vec<_>>();

        Ok(users)
    }
}
