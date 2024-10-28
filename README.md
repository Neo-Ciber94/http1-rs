# http1 

A zero dependencies implementation of the HTTP 1.1 protocol in rust.

## TODO

### Types

- Add ip address information, maybe just inject the IpAddress or a Connection struct

### Misc
- Web Sockets?
- Separate read_request and write_response so only use io::Read and io::Write
- RouteInfo to allow inject the current route information

### Fixes
- FromRequest should return a custom error, by default we just return 500 but some can return 400 like Path