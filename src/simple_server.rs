use crate::tokens::token_store::TokenStore;
use anyhow::Result;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
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
                Ok(stream) => self.serve_connection(stream),
                Err(e) => eprintln!("Error accepting connection: {}", e),
            }
        }
        Ok(())
    }

    fn serve_connection(&self, stream: TcpStream) {
        let auth_token = self.extract_auth_token(&stream);
        // if no auth header, cannot be valid
        match auth_token {
            Some(auth_token) => match self.token_store.contains_token(&auth_token) {
                Ok(result) => match result {
                    true => self.respond(stream, HttpResponse::Ok),
                    false => self.respond(stream, HttpResponse::Unauthorised),
                },
                Err(e) => eprintln!("Failed to serve connection: {}", e),
            },
            None => self.respond(stream, HttpResponse::Unauthorised),
        }
    }

    fn extract_auth_token(&self, mut stream: &TcpStream) -> Option<String> {
        let buf_reader = BufReader::new(&mut stream);
        buf_reader
            .lines()
            .map(|result| result.unwrap())
            .find(|line| line.starts_with("Authorization: Bearer"))
            .map(|line| line["Authorization: Bearer ".len()..].to_string())
    }

    fn respond(&self, mut stream: TcpStream, response: HttpResponse) {
        match stream.write_all(response.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to write response to stream: {}", e);
            }
        }
    }
}
