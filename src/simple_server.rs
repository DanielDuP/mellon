use crate::tokens::token_store::TokenStore;
use anyhow::Result;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    time::Duration,
};

enum HttpResponse {
    Ok,
    Unauthorised,
}

impl HttpResponse {
    fn as_str(&self) -> &str {
        match self {
            HttpResponse::Ok => "HTTP/1.1 200 OK\r\n\r\n",
            HttpResponse::Unauthorised => "HTTP/1.1 401 UNAUTHORISED\r\n\r\n",
        }
    }

    fn as_bytes(&self) -> &[u8] {
        return self.as_str().as_bytes();
    }
}

pub struct MellonServer {
    token_store: TokenStore,
    host_name: String,
}

impl MellonServer {
    pub fn serve(host_name: String, token_store: TokenStore) -> Result<()> {
        let server = MellonServer {
            token_store,
            host_name,
        };
        server.listen()
    }

    fn listen(&self) -> Result<()> {
        let listener = match TcpListener::bind(&self.host_name) {
            Ok(listener) => listener,
            Err(e) => {
                eprintln!("Failed to bind to {}: {}", self.host_name, e);
                return Err(e.into());
            }
        };

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => self
                    .serve_connection(stream)
                    .unwrap_or_else(|e| eprintln!("Failed to serve request {}", e)),
                Err(e) => eprintln!("Error accepting connection: {}", e),
            }
        }
        Ok(())
    }

    fn serve_connection(&self, stream: TcpStream) -> Result<()> {
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        let auth_token = self.extract_auth_token(&stream)?;
        // if no auth header, cannot be valid
        match auth_token {
            // i.e. we have found the auth token from the headers
            // now we just test it against the token store
            Some(auth_token) => match self.token_store.contains_token(&auth_token)? {
                true => self.respond(stream, HttpResponse::Ok)?,
                false => self.respond(stream, HttpResponse::Unauthorised)?,
            },
            // No auth token obviously means request cannot be authorized
            None => self.respond(stream, HttpResponse::Unauthorised)?,
        }
        Ok(())
    }

    fn extract_auth_token(&self, stream: &TcpStream) -> Result<Option<String>> {
        let buf_reader = BufReader::new(stream);
        for line in buf_reader.lines() {
            match line {
                Ok(line) => {
                    if let Some(token) = line.strip_prefix("Authorization: Bearer ") {
                        return Ok(Some(token.to_string()));
                    }
                    if line.is_empty() {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    return Err(anyhow::anyhow!(
                        "Connection timed out while reading headers"
                    ));
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(None)
    }

    fn respond(&self, mut stream: TcpStream, response: HttpResponse) -> Result<()> {
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;
        stream.write_all(response.as_bytes())?;
        Ok(())
    }
}
