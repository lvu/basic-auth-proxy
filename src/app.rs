use crate::{basic, err, oidc};
use http_body_util::Empty;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};

pub struct App {
    oidc_client: oidc::OidcClient,
}

type EmptyResponse = Response<Empty<Bytes>>;

impl App {
    pub async fn new(config: &crate::Config) -> Self {
        let oidc_client = oidc::OidcClient::new(
            &config.oidc_issuer,
            &config.oidc_client_id,
            &config.oidc_client_secret,
            config.groups_claim.clone(),
            config.additional_scopes.clone(),
        )
        .await
        .unwrap();
        Self {
            oidc_client,
        }
    }

    async fn auth(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<EmptyResponse, err::ProxyError> {
        let credentials = basic::parse_basic_auth(req.headers())
            .map_err(|e| err::ProxyError::from_source(e, StatusCode::UNAUTHORIZED))?;
        println!("Authorizing user: {:?}", credentials.username);
        let user_info = self.oidc_client.get_user_info(
            &credentials.username,
            &credentials.password,
        )
        .await?;
        let mut response = Response::builder()
            .status(StatusCode::OK)
            .header("X-Auth-Request-User", user_info.id);
        if let Some(email) = user_info.email {
            response = response.header("X-Auth-Request-Email", email);
        }
        if let Some(preferred_username) = user_info.preferred_username {
            response = response.header("X-Auth-Request-Preferred-Username", preferred_username);
        }
        if !user_info.groups.is_empty() {
            response = response.header("X-Auth-Request-Groups", user_info.groups.join(","));
        }
        Ok(response.body(Empty::new()).map_err(|e| err::ProxyError::from_source(e.into(), StatusCode::INTERNAL_SERVER_ERROR))?)
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
