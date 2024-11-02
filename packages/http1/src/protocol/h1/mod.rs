mod request;
mod response;

use std::{io::ErrorKind, net::TcpStream};

use crate::{handler::RequestHandler, server::Config};

/**
 * Handles and send a response to a HTTP1 request.
 */
pub fn handle_incoming<H>(handler: &H, config: &Config, stream: TcpStream) -> std::io::Result<()>
where
    H: RequestHandler + Send + Sync + 'static,
{
    let mut writer = stream.try_clone()?;
    let request = request::read_request(stream)?;
    let response = handler.handle(request);

    match response::write_response(response, &mut writer, config) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == ErrorKind::ConnectionAborted => Ok(()),
        Err(err) => Err(err),
    }
}
