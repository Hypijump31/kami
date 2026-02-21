//! HTTP fetch tool — KAMI guest component using WASI HTTP outgoing.
//!
//! Makes a real HTTP GET via the host's `wasi:http/outgoing-handler`.
//! The host enforces `net_allow_list` — blocked hosts return an error.

wit_bindgen::generate!({
    world: "http-fetch-tool",
    path: "wit",
    generate_all,
});

use exports::kami::tool::tool::Guest;
use wasi::http::outgoing_handler;
use wasi::http::types::{Fields, Method, OutgoingRequest, RequestOptions, Scheme};

struct HttpFetchTool;

impl Guest for HttpFetchTool {
    fn run(input: String) -> Result<String, String> {
        let url = parse_url(&input)?;
        let body = http_get(&url)?;
        Ok(body)
    }

    fn describe() -> String {
        r#"{"name":"http-fetch","description":"Fetches a URL via WASI HTTP","version":"0.1.0"}"#
            .to_string()
    }
}

export!(HttpFetchTool);

/// Parses the `url` field from JSON input.
fn parse_url(input: &str) -> Result<String, String> {
    let val: serde_json::Value =
        serde_json::from_str(input).map_err(|e| format!("invalid JSON: {e}"))?;
    val["url"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "missing 'url' field".to_string())
}

/// Makes an HTTP GET request via WASI outgoing handler.
fn http_get(raw_url: &str) -> Result<String, String> {
    let (scheme, authority, path) = split_url(raw_url)?;

    let headers = Fields::new();
    let request = OutgoingRequest::new(headers);
    request
        .set_method(&Method::Get)
        .map_err(|()| "set method failed".to_string())?;
    request
        .set_scheme(Some(&scheme))
        .map_err(|()| "set scheme failed".to_string())?;
    request
        .set_authority(Some(&authority))
        .map_err(|()| "set authority failed".to_string())?;
    request
        .set_path_with_query(Some(&path))
        .map_err(|()| "set path failed".to_string())?;

    let future = outgoing_handler::handle(request, None::<RequestOptions>)
        .map_err(|e| format!("HTTP request denied: {e:?}"))?;

    let pollable = future.subscribe();
    pollable.block();

    let response = future
        .get()
        .ok_or("no response")?
        .map_err(|()| "future polled twice".to_string())?
        .map_err(|e| format!("HTTP error: {e:?}"))?;

    let body = response
        .consume()
        .map_err(|()| "consume failed".to_string())?;
    let stream = body
        .stream()
        .map_err(|()| "stream failed".to_string())?;

    let mut bytes = Vec::new();
    loop {
        match stream.read(4096) {
            Ok(chunk) if chunk.is_empty() => break,
            Ok(chunk) => {
                bytes.extend_from_slice(&chunk);
                if bytes.len() >= 8192 {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    String::from_utf8(bytes).map_err(|e| format!("invalid UTF-8: {e}"))
}

/// Splits a URL into (Scheme, authority, path+query).
fn split_url(url: &str) -> Result<(Scheme, String, String), String> {
    let (scheme, rest) = if let Some(s) = url.strip_prefix("https://") {
        (Scheme::Https, s)
    } else if let Some(s) = url.strip_prefix("http://") {
        (Scheme::Http, s)
    } else {
        return Err(format!("unsupported scheme in '{url}'"));
    };

    let (authority, path) = if let Some(slash) = rest.find('/') {
        (rest[..slash].to_string(), rest[slash..].to_string())
    } else {
        (rest.to_string(), "/".to_string())
    };

    Ok((scheme, authority, path))
}
