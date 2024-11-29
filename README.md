# http1

A zero dependencies implementation of the HTTP 1.1 protocol in rust.

## TODO

- [x] Web client
- [x] Web Sockets
- [ ] Allow to pre render `App`
- [ ] Add a AsciiString to restrict headers and other types to it
- [x] Allow use extractors in any order my making the body `Mutex<Option<Body>>>` and only keep `FromRequestRef` or rename it to `FromRequest`. (The solution was the Payload)

## FIXME

- [ ] Mime type implementation is not correct, MUST be case insensitive: <https://httpwg.org/specs/rfc9110.html#media.type>
