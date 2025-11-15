// HTTP wrapper for Icarus MCP Server
// Exposes the stdio-based MCP server over HTTP for Claude Code web integration

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::{Child, Command as TokioCommand};
use tokio::sync::Mutex;

#[derive(Debug)]
struct McpServer {
    process: Child,
}

impl McpServer {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let process = TokioCommand::new(std::env::current_exe()?)
            .arg("--stdio") // Assuming icarus-mcp supports --stdio flag
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Self { process })
    }

    async fn send_request(&mut self, request: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Write request to MCP server stdin
        if let Some(stdin) = &mut self.process.stdin {
            stdin.write_all(request.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await?;
        }

        // Read response from MCP server stdout
        if let Some(stdout) = &mut self.process.stdout {
            let mut reader = tokio::io::BufReader::new(stdout);
            let mut response = String::new();
            reader.read_line(&mut response).await?;
            Ok(response)
        } else {
            Err("No stdout available".into())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let port = if args.len() > 1 {
        args[1].parse::<u16>().unwrap_or(3000)
    } else {
        3000
    };

    println!("🚀 Starting Icarus MCP HTTP Server on port {}", port);

    // Initialize MCP server (stdio-based)
    let server = Arc::new(Mutex::new(McpServer::new().await?));

    // Setup HTTP server using hyper
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Method, Request, Response, Server, StatusCode};

    let make_svc = make_service_fn(move |_conn| {
        let server = Arc::clone(&server);
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let server = Arc::clone(&server);
                handle_request(req, server)
            }))
        }
    });

    let addr = ([0, 0, 0, 0], port).into();
    let server_builder = Server::bind(&addr).serve(make_svc);

    println!("✅ Icarus MCP Server ready at http://0.0.0.0:{}", port);
    println!("📝 Add to Claude Code with:");
    println!("   claude mcp add --transport http icarus http://localhost:{}", port);
    println!();

    server_builder.await?;

    Ok(())
}

async fn handle_request(
    req: Request<Body>,
    server: Arc<Mutex<McpServer>>,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Health check endpoint
        (&Method::GET, "/health") => Ok(Response::new(Body::from(
            r#"{"status":"ok","service":"icarus-mcp"}"#,
        ))),

        // MCP endpoint
        (&Method::POST, "/mcp") | (&Method::POST, "/") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
            let request_str = String::from_utf8_lossy(&body_bytes);

            let mut server = server.lock().await;
            match server.send_request(&request_str).await {
                Ok(response) => {
                    let mut res = Response::new(Body::from(response));
                    res.headers_mut().insert(
                        hyper::header::CONTENT_TYPE,
                        hyper::header::HeaderValue::from_static("application/json"),
                    );
                    Ok(res)
                }
                Err(e) => {
                    let error_json = format!(r#"{{"error":"{}"}}"#, e);
                    let mut res = Response::new(Body::from(error_json));
                    *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    Ok(res)
                }
            }
        }

        // 404 for everything else
        _ => {
            let mut res = Response::new(Body::from("Not Found"));
            *res.status_mut() = StatusCode::NOT_FOUND;
            Ok(res)
        }
    }
}
