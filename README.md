# http1 

A zero dependencies implementation of the HTTP 1.1 protocol in rust.

## TODO

### Types

- `Query<T>`
- `Form<T>`
- `Multipart<T>`
- Add types that implement FromRequest that can be created from headers like: GetHeader<Authorization>, TypedHeader<Authrization>, H<Authorization>, FromHeader<Authorization>
- Add Host information from the connection reading it from the headers, that can be a GetHeader<Host>

### Misc
- Web Sockets?
- Middleware to read static files from a path ***

### Fixes
- FromRequest should return a custom error, by default we just return 500 but some can return 400 like Path