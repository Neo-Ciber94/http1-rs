use http1::{method::Method, server::Server};
use http1_web::{app::App, html::Html, middleware::cors::Cors};

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let t1 = std::thread::spawn(move || backend().expect("failed to start backend"));
    let t2 = std::thread::spawn(move || frontend().expect("failed to start frontend"));

    t1.join().unwrap();
    t2.join().unwrap();
    Ok(())
}

fn backend() -> std::io::Result<()> {
    let app = App::new()
        .middleware(
            Cors::builder()
                .allow_methods(vec![Method::GET, Method::POST])
                .allow_origins(vec!["http://localhost:3456"])
                .build(),
        )
        .get("/api/users", || "All Users")
        .post("/api/users", || "Create User");

    Server::new()
        .on_ready(|addr| println!("Backend listening on http://localhost:{}", addr.port()))
        .listen("0.0.0.0:5000", app)
}

fn frontend() -> std::io::Result<()> {
    let app = App::new().get("/", || {
        Html(
            r#"
        <html>
            <head>
                <title>User List</title>
                <script>
                    window.onload = () => {
                        const userDiv = document.getElementById('user-list');
                        
                        fetch('http://localhost:5000/api/users')
                            .then(response => response.text()) 
                            .then(text => {
                                userDiv.textContent = text;
                            })
                            .catch(error => {
                                console.error('Error fetching users:', error);
                                userDiv.textContent = 'Failed to load users'; 
                            });
                    };
                </script>
            </head>
            <body>
                <h1>Users</h1>
                <div id="user-list">
                   Loading...
                </div>
            </body>
        </html>
        "#,
        )
    });

    Server::new()
        .on_ready(|addr| println!("Frontend listening on http://localhost:{}", addr.port()))
        .listen("0.0.0.0:3456", app)
}
