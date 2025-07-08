use std::error::Error;

#[derive(Debug, Clone)]
pub struct ProxyError {
    message: String,
    status: hyper::StatusCode,
}

impl ProxyError {
    pub fn new(message: String, status: hyper::StatusCode) -> Self {
        Self {
            message,
            status,
        }
    }

    pub fn from_source(source: Box<dyn Error>, status: hyper::StatusCode) -> Self {
        Self {
            message: source.to_string(),
            status,
        }
    }

    pub fn status(&self) -> hyper::StatusCode {
        self.status
    }
}

impl Error for ProxyError {}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status, self.message)
    }
}

impl From<Box<dyn Error>> for ProxyError {
    fn from(e: Box<dyn Error>) -> Self {
        match e.downcast::<ProxyError>() {
            Ok(e) => *e,
            Err(e) => Self::from_source(e, hyper::StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proxy_result() -> Result<(), Box<dyn Error>> {
        i32::from_str_radix("foo", 10)
            .map_err(|e| ProxyError::from_source(e.into(), hyper::StatusCode::UNAUTHORIZED))?;
        Ok(())
    }

    #[test]
    fn test_proxy_downcast() {
        let e = ProxyError::new("test".to_string(), hyper::StatusCode::UNAUTHORIZED);
        let e2: Box<dyn Error> = e.into();
        let e3: ProxyError = e2.into();
        assert_eq!(e3.status(), hyper::StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_proxy_result() {
        let r = proxy_result();
        assert!(r.is_err());
        let e = r.unwrap_err();
        let e2 = e.downcast::<ProxyError>();
        assert!(e2.is_ok());
        assert!(e2.unwrap().status() == hyper::StatusCode::UNAUTHORIZED);
    }
}
