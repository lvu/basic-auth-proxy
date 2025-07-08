use base64::prelude::*;
use hyper::header::HeaderMap;
use std::error;
use std::io;

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

pub fn parse_basic_auth(headers: &HeaderMap) -> Result<Credentials, Box<dyn error::Error>> {
    let auth = headers
        .get("Authorization")
        .ok_or(io::Error::new(io::ErrorKind::InvalidInput, "No basic auth"))?;
    let auth = auth.to_str()?;
    let (scheme, payload) = auth.split_at(6);
    if scheme != "Basic " {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("Unsupported auth scheme: {}", scheme),
        )));
    }
    let decoded = BASE64_STANDARD.decode(payload)?;
    let decoded = String::from_utf8(decoded)?;
    let (username, password) = decoded.split_once(":").ok_or(Box::new(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Invalid basic auth",
    )))?;
    let credentials = Credentials {
        username: username.to_string(),
        password: password.to_string(),
    };
    Ok(credentials)
}
