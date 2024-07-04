# Mellon

Mellon is a straightforward, Rust-based authentication server that takes its
inspiration from the iconic riddle on the Doors of Durin
in J.R.R. Tolkien's "The Lord of the Rings."

Simply speak your bearer token and enter.

It can be configured to work with standalone webservers, such as
Apache and NGINX to guard access to processes lacking specific auth mechanisms of their own.

## Disclaimer

Mellon is designed (barely) for serving a very unique and specific use case - namely gating access to another web-based service
which has no authentication whatsoever and where it would prove impractical, tiresome or boring to implement said security mechanisms
in that service itself.
Mellon is designed to be used in conjunction with a reverse proxy or similar and will be of no use on its own.

It should probably not be used - and, even in the very specific use case described above, there exist more robust, extensible solutions.

If your application requires any form of authentication beyond the use of an Authorization header with a bearer token, Mellon is not for you.
If you anticipate the need to manage a large number of different clients each requiring unique tokens, Mellon is not for you.

Even if you think Mellon is appropriate for your use case - Mellon still probably is not for you.

## Features

- Easy integration with web servers like NGINX or Apache
- Simple bearer token authentication
- CLI interface for generating, listing and rescinding tokens on the host machine
- Immediate response with HTTP status codes for authentication status

## Getting Started

### Prerequisites

- A working installation of NGINX or Apache

### Installation

1. Clone the Mellon repository from GitHub.
2. Navigate to the project directory.
3. Install dependencies if required by running `cargo check`.
4. Compile the project using `cargo build --release`.
5. Create and configure your token list as needed.
6. Configure Mellon to run as a background process
7. Set up your web server to interface with the Mellon

### Configuration

To set up your service with rudimentary authentication using nginx, you'll need to add specific configurations to your nginx server block. Here's how to do it:

#### Setup Your Main Service

1. Direct requests to the auth service for authentication by using the `auth_request` directive.
2. Use `proxy_pass` to forward the request to your actual service once authenticated.

Your nginx server block for the main service should look something like this:

```nginx
server {
    listen 80;

    location / {
        auth_request /auth; # Pass requests to the auth service
        proxy_pass http://localhost:8080; # Your actual service's URL
    }
}
```

#### Configure Auth as an Internal Location

The auth service should be configured as an internal location within nginx to handle authentication requests. The configuration ensures that the authentication requests are not directly accessible from the outside.

Add the following to your nginx configuration:

```nginx
location = /auth {
    internal; # Makes this location accessible only for internal requests
    proxy_pass http://localhost:9090/auth; # mellon default URL

    proxy_pass_request_body off; # Do not send the body to the auth service
    proxy_set_header Content-Length ""; # Clear the content length header
    proxy_set_header X-Original-URI $request_uri; # Pass the original URI
    proxy_set_header X-Api-Key $http_x_api_key; # Pass the API key if provided
    proxy_method GET; # Change the request method to GET for the auth service
}
```

#### How It Works

- When a request hits your main service at `/`, nginx first checks with the auth service using the `/auth` location.
- The auth service examines the `Authorization` header and ensures the token is valid.
- If the auth service returns a success status (HTTP 2xx), nginx then forwards the request to your actual service.
- If the auth service returns an error (HTTP 4xx or 5xx), nginx rejects the request and returns the error to the client.

Make sure to replace `http://localhost:8080` and `http://localhost:9090/auth` with the actual URLs of your main service and auth service, respectively. Also, ensure your auth service properly checks the `Authorization` header and responds with the appropriate HTTP status codes.

## Usage

### General

```bash
Usage: mellon <COMMAND>
```

**Commands:**

- `serve` - Starts the auth server
- `token` - Manage tokens by adding or removing
- `help` - Print this message or the help of the given subcommand(s)

**Options:**

- `-h`, `--help` - Print help (see a summary with `-h`)
- `-V`, `--version` - Print version

### Token Management

```bash
Usage: mellon token <COMMAND>
```

**Commands:**

- `add` - Add a new token
- `rescind` - Revoke an existing token by its label
- `list` - List all tokens previously issued
- `help` - Print this message or the help of the given subcommand(s)

**Options:**

- `-h`, `--help` - Print help

## API Reference

- `GET /auth` - Endpoint to check for authentication.

## License

This project is licensed under the BSD 3-Clause License. For more details, see the [LICENSE](LICENSE) file in the repository.
