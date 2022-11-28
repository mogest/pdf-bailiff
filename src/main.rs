use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use std::env;
use std::io::prelude::*;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{convert::Infallible, net::SocketAddr};
use timeout_readwrite::{TimeoutReader, TimeoutWriter};

const MAX_PAYLOAD_LENGTH: usize = 5 * 1048576;
const TIMEOUT: Duration = Duration::new(5, 0);

async fn handle(request: Request<Body>) -> Result<Response<Body>, hyper::http::Error> {
    if request.method() != Method::POST {
        return Response::builder()
            .status(hyper::StatusCode::METHOD_NOT_ALLOWED)
            .body(Body::from("Method not allowed"));
    }

    let body = match hyper::body::to_bytes(request.into_body()).await {
        Err(_) => {
            return Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal server error reading http body"))
        }

        Ok(value) => value,
    };

    if body.len() > MAX_PAYLOAD_LENGTH {
        return Response::builder()
            .status(hyper::StatusCode::PAYLOAD_TOO_LARGE)
            .body(Body::from("Payload too large"));
    }

    let process = match Command::new("pdftotext")
        .args(["-", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Err(_) => {
            return Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal server error executing binary"))
        }
        Ok(process) => process,
    };

    match TimeoutWriter::new(process.stdin.unwrap(), TIMEOUT).write_all(&body) {
        Err(_) => {
            return Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal server error writing to pipe"))
        }
        Ok(_) => (),
    }

    let stdout = process.stdout.unwrap();
    let mut s = String::new();
    let mut reader = TimeoutReader::new(stdout, TIMEOUT);

    match reader.read_to_string(&mut s) {
        Err(_) => Response::builder()
            .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Internal server error reading from pipe")),
        Ok(_) => Ok(Response::new(s.into())),
    }
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT")
        .map_err(|_| ())
        .and_then(|string| string.parse::<u16>().map_err(|_| ()))
        .unwrap_or(3500);

    println!("starting http server on port {}", port);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
