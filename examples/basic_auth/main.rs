use http1::server::Server;
use http1_web::{
    app::App,
    header::{BasicAuth, GetHeader},
    html, ErrorResponse, HttpResponse, WWWAuthenticate,
};

fn main() -> std::io::Result<()> {
    let app: App = App::new().get("/", home);

    Server::new()
        .on_ready(|addr| println!("Listening on http://localhost:{}", addr.port()))
        .listen("0.0.0.0:3000", app)
}

fn home(auth: Option<GetHeader<BasicAuth>>) -> Result<HttpResponse, ErrorResponse> {
    let www_auth = WWWAuthenticate::new("my-realm");

    match auth {
        Some(GetHeader(auth)) => {
            if !authenticate(&auth) {
                return Ok(HttpResponse::from_response(www_auth.clone()).finish());
            }

            let html = html::html(|| {
                html::head(|| {
                    html::attr("charset", "utf-8");
                    html::title("Welcome!");
                });

                html::body(|| {
                    html::h1(format!("Welcome {}!", auth.username));
                });
            });

            Ok(HttpResponse::from_response(html).finish())
        }
        None => Ok(HttpResponse::from_response(www_auth.clone()).finish()),
    }
}

#[derive(Debug)]
struct User {
    username: &'static str,
    password: &'static str,
}

static USERS: &[User] = &[
    User {
        username: "ErenYeager",
        password: "TitanSlayer123",
    },
    User {
        username: "TanjiroKamado",
        password: "DemonHunter456",
    },
    User {
        username: "YujiItadori",
        password: "CursedPower789",
    },
    User {
        username: "IzukuMidoriya",
        password: "PlusUltra321",
    },
    User {
        username: "NezukoKamado",
        password: "DemonSister101",
    },
    User {
        username: "GojoSatoru",
        password: "InfinityMaster111",
    },
];

fn authenticate(auth: &BasicAuth) -> bool {
    USERS
        .iter()
        .any(|user| user.username == auth.username && user.password == auth.password)
}
