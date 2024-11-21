# http1

A zero dependencies implementation of the HTTP 1.1 protocol in rust.

## TODO

- [x] Web client
- [x] Web Sockets
- [ ] Allow to pre render `App`
- [ ] Allow use extractors in any order my making the body `Mutex<Option<Body>>>` and only keep `FromRequestRef` or rename it to `FromRequest`.
