use std::error;

use openidconnect::core;
use openidconnect::{
    AdditionalClaims, ClientId, ClientSecret, EmptyExtraTokenFields, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, IdTokenClaims, IdTokenFields, IdTokenVerifier, IssuerUrl, Nonce,
    OAuth2TokenResponse, ResourceOwnerPassword, ResourceOwnerUsername, RevocationErrorResponseType,
    Scope, StandardErrorResponse, StandardTokenIntrospectionResponse, StandardTokenResponse,
    TokenResponse, UserInfoClaims,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::err;

#[derive(Debug)]
pub struct OidcUserInfo {
    pub id: String,
    pub email: Option<String>,
    pub preferred_username: Option<String>,
    pub groups: Vec<String>,
}

pub struct OidcClient {
    client: InternalOidcClient,
    http_client: reqwest::Client,
    groups_claim: Option<String>,
    additional_scopes: Vec<String>,
}

impl OidcClient {
    pub async fn new(
        issuer: &str,
        client_id: &str,
        client_secret: &str,
        groups_claim: Option<String>,
        additional_scopes: Vec<String>,
    ) -> Result<Self, Box<dyn error::Error>> {
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let provider_metadata = core::CoreProviderMetadata::discover_async(
            IssuerUrl::new(issuer.to_string())?,
            &http_client,
        )
        .await?;
        let client = InternalOidcClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(client_id.to_string()),
            Some(ClientSecret::new(client_secret.to_string())),
        );
        Ok(Self {
            client,
            http_client,
            groups_claim,
            additional_scopes,
        })
    }

    pub async fn get_user_info(
        &self,
        username: &str,
        password: &str,
    ) -> Result<OidcUserInfo, Box<dyn error::Error>> {
        let username = ResourceOwnerUsername::new(username.to_string());
        let password = ResourceOwnerPassword::new(password.to_string());

        let mut password_req = self
            .client
            .exchange_password(&username, &password)?
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()));
        password_req = self
            .additional_scopes
            .iter()
            .fold(password_req, |req, scope| {
                req.add_scope(Scope::new(scope.to_string()))
            });

        let resp = password_req
            .request_async(&self.http_client)
            .await
            .map_err(|e| err::ProxyError::from_source(e.into(), hyper::StatusCode::UNAUTHORIZED))?;
        let ver: IdTokenVerifier<core::CoreJsonWebKey> =
            IdTokenVerifier::new_insecure_without_verification();
        let claims = resp
            .id_token()
            .map(|t| t.claims(&ver, insecure_nonce_verifier).unwrap());
        if claims.is_some() {
            let user_info = self.parse_id_token_claims(&claims.unwrap())?;
            if user_info.is_some() {
                return Ok(user_info.unwrap());
            }
        }
        let access_token = resp.access_token().clone();
        let info_req = self.client.user_info(access_token, None)?;
        let resp: UserInfoClaims<DynamicAdditionalClaims, core::CoreGenderClaim> =
            info_req.request_async(&self.http_client).await?;
        self.parse_user_info_claims(&resp)
    }

    fn parse_id_token_claims(
        &self,
        claims: &IdTokenClaims<DynamicAdditionalClaims, core::CoreGenderClaim>,
    ) -> Result<Option<OidcUserInfo>, Box<dyn error::Error>> {
        let mut groups: Vec<String> = Vec::new();
        if let Some(groups_claim_name) = &self.groups_claim {
            match claims.additional_claims().0.get(groups_claim_name) {
                Some(groups_claim) => {
                    groups = serde_json::from_value(groups_claim.clone())?;
                }
                _ => {
                    return Ok(None);
                }
            }
        }
        Ok(Some(OidcUserInfo {
            id: claims.subject().to_string(),
            email: claims.email().map(|e| e.to_string()),
            preferred_username: claims.preferred_username().map(|u| u.to_string()),
            groups,
        }))
    }

    fn parse_user_info_claims(
        &self,
        claims: &UserInfoClaims<DynamicAdditionalClaims, core::CoreGenderClaim>,
    ) -> Result<OidcUserInfo, Box<dyn error::Error>> {
        let groups: Vec<String> = self
            .groups_claim
            .as_ref()
            .and_then(|gn| {
                claims
                    .additional_claims()
                    .0
                    .get(gn)
                    .map(|g| serde_json::from_value(g.clone()))
            })
            .unwrap_or(Ok(Vec::new()))?;
        Ok(OidcUserInfo {
            id: claims.subject().to_string(),
            email: claims.email().map(|e| e.to_string()),
            preferred_username: claims.preferred_username().map(|u| u.to_string()),
            groups,
        })
    }
}

type InternalOidcClient = openidconnect::Client<
    DynamicAdditionalClaims,
    core::CoreAuthDisplay,
    core::CoreGenderClaim,
    core::CoreJweContentEncryptionAlgorithm,
    core::CoreJsonWebKey,
    core::CoreAuthPrompt,
    StandardErrorResponse<core::CoreErrorResponseType>,
    StandardTokenResponse<
        IdTokenFields<
            DynamicAdditionalClaims,
            EmptyExtraTokenFields,
            core::CoreGenderClaim,
            core::CoreJweContentEncryptionAlgorithm,
            core::CoreJwsSigningAlgorithm,
        >,
        core::CoreTokenType,
    >,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, core::CoreTokenType>,
    core::CoreRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DynamicAdditionalClaims(Map<String, Value>);

impl AdditionalClaims for DynamicAdditionalClaims {}

fn insecure_nonce_verifier(_: Option<&Nonce>) -> Result<(), String> {
    Ok(())
}
