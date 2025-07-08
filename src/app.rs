use std::sync::Arc;
use std::time::{Duration, Instant};

use http_body_util::Empty;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use quick_cache::sync::Cache;

use crate::{basic, err, oidc};

pub struct App {
    oidc_client: oidc::OidcClient,
    cache: Cache<basic::Credentials, CacheEntry>,
    cache_ttl: Duration,
}

impl App {
    pub async fn new(config: &crate::Config) -> Self {
        let oidc_client = oidc::OidcClient::new(
            &config.issuer,
            &config.client_id,
            &config.client_secret,
            config.groups_claim.clone(),
            config.additional_scopes.clone(),
        )
        .await
        .unwrap();
        Self {
            oidc_client,
            cache: Cache::new(config.cache_max_size),
            cache_ttl: Duration::from_secs(config.cache_ttl_seconds),
        }
    }

    async fn get_user_info(
        &self,
        credentials: &basic::Credentials,
    ) -> Arc<Result<oidc::OidcUserInfo, err::ProxyError>> {
        let mut entry = self.cache.get_value_or_guard_async(credentials).await;
        if let Ok(value) = &entry {
            if value.expires_at < Instant::now() {
                self.cache.remove(credentials);
                entry = self.cache.get_value_or_guard_async(credentials).await;
            }
        }
        match entry {
            Ok(value) => value.user_info.clone(),
            Err(g) => {
                println!("Authorizing user: {:?}", credentials.username);
                let result: Result<oidc::OidcUserInfo, err::ProxyError> = self
                    .oidc_client
                    .get_user_info(&credentials.username, &credentials.password)
                    .await
                    .map_err(|e| e.into());
                let result = Arc::new(result);
                let _ = g.insert(CacheEntry {
                    user_info: result.clone(),
                    expires_at: Instant::now() + self.cache_ttl,
                });
                result
            }
        }
    }

    async fn auth(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<EmptyResponse, err::ProxyError> {
        let credentials = basic::parse_basic_auth(req.headers())
            .map_err(|e| err::ProxyError::from_source(e, StatusCode::UNAUTHORIZED))?;
        match &*self.get_user_info(&credentials).await {
            Err(e) => Err(e.clone()),
            Ok(user_info) => {
                let mut response = Response::builder()
                    .status(StatusCode::OK)
                    .header("X-Auth-Request-User", &user_info.id);
                if let Some(email) = &user_info.email {
                    response = response.header("X-Auth-Request-Email", email);
                }
                if let Some(preferred_username) = &user_info.preferred_username {
                    response =
                        response.header("X-Auth-Request-Preferred-Username", preferred_username);
                }
                if !user_info.groups.is_empty() {
                    response = response.header("X-Auth-Request-Groups", user_info.groups.join(","));
                }
                Ok(response.body(Empty::new()).map_err(|e| {
                    err::ProxyError::from_source(e.into(), StatusCode::INTERNAL_SERVER_ERROR)
                })?)
            }
        }
    }

    pub async fn handle_auth(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> hyper::http::Result<EmptyResponse> {
        match self.auth(req).await {
            Ok(resp) => Ok(resp),
            Err(e) => {
                println!("Error: {}", e);
                Ok(Response::builder().status(e.status()).body(Empty::new())?)
            }
        }
    }
}

type EmptyResponse = Response<Empty<Bytes>>;

#[derive(Clone)]
struct CacheEntry {
    user_info: Arc<Result<oidc::OidcUserInfo, err::ProxyError>>,
    expires_at: Instant,
}
