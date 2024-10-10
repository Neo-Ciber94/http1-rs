# http1 

A zero dependencies implementation of the HTTP 1.1 protocol in rust.

## TODO

### Types

- `Query<T>`
- `Form<T>`
- `Multipart<T>`

### Fixes
- FromRequest should return a custom error, by default we just return 500 but some can return 400 like Path