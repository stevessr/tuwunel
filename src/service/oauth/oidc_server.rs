use std::{sync::Arc, time::{Duration, SystemTime}};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as b64};
use ring::{rand::SystemRandom, signature::{self, EcdsaKeyPair, KeyPair}};
use ruma::OwnedUserId;
use serde::{Deserialize, Serialize};
use tuwunel_core::{Err, Result, err, info, jwt, utils};
use tuwunel_database::{Cbor, Deserialized, Map};

const AUTH_CODE_LENGTH: usize = 64;
const OIDC_CLIENT_ID_LENGTH: usize = 32;
const AUTH_CODE_LIFETIME: Duration = Duration::from_secs(600);
const AUTH_REQUEST_LIFETIME: Duration = Duration::from_secs(600);
const SIGNING_KEY_DB_KEY: &str = "oidc_signing_key";

pub struct OidcServer { db: Data, signing_key_der: Vec<u8>, jwk: serde_json::Value, key_id: String }

struct Data { oidc_signingkey: Arc<Map>, oidcclientid_registration: Arc<Map>, oidccode_authsession: Arc<Map>, oidcreqid_authrequest: Arc<Map> }

#[derive(Debug, Deserialize)]
pub struct DcrRequest {
	pub redirect_uris: Vec<String>, pub client_name: Option<String>, pub client_uri: Option<String>,
	pub logo_uri: Option<String>, #[serde(default)] pub contacts: Vec<String>,
	pub token_endpoint_auth_method: Option<String>, pub grant_types: Option<Vec<String>>,
	pub response_types: Option<Vec<String>>, pub application_type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OidcClientRegistration {
	pub client_id: String, pub redirect_uris: Vec<String>, pub client_name: Option<String>,
	pub client_uri: Option<String>, pub logo_uri: Option<String>, pub contacts: Vec<String>,
	pub token_endpoint_auth_method: String, pub grant_types: Vec<String>, pub response_types: Vec<String>,
	pub application_type: Option<String>, pub registered_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthCodeSession {
	pub code: String, pub client_id: String, pub redirect_uri: String, pub scope: String,
	pub state: Option<String>, pub nonce: Option<String>, pub code_challenge: Option<String>,
	pub code_challenge_method: Option<String>, pub user_id: OwnedUserId,
	pub created_at: SystemTime, pub expires_at: SystemTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OidcAuthRequest {
	pub client_id: String, pub redirect_uri: String, pub scope: String, pub state: Option<String>,
	pub nonce: Option<String>, pub code_challenge: Option<String>, pub code_challenge_method: Option<String>,
	pub created_at: SystemTime, pub expires_at: SystemTime,
}

#[derive(Serialize, Deserialize)]
struct SigningKeyData { key_der: Vec<u8>, key_id: String }

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderMetadata {
	pub issuer: String, pub authorization_endpoint: String, pub token_endpoint: String,
	pub registration_endpoint: Option<String>, pub revocation_endpoint: Option<String>, pub jwks_uri: String,
	pub userinfo_endpoint: Option<String>, pub account_management_uri: Option<String>,
	pub account_management_actions_supported: Option<Vec<String>>, pub response_types_supported: Vec<String>,
	pub response_modes_supported: Option<Vec<String>>, pub grant_types_supported: Option<Vec<String>>,
	pub code_challenge_methods_supported: Option<Vec<String>>, pub token_endpoint_auth_methods_supported: Option<Vec<String>>,
	pub scopes_supported: Option<Vec<String>>, pub subject_types_supported: Option<Vec<String>>,
	pub id_token_signing_alg_values_supported: Option<Vec<String>>, pub prompt_values_supported: Option<Vec<String>>,
	pub claim_types_supported: Option<Vec<String>>, pub claims_supported: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdTokenClaims {
	pub iss: String, pub sub: String, pub aud: String, pub exp: u64, pub iat: u64,
	#[serde(skip_serializing_if = "Option::is_none")] pub nonce: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")] pub at_hash: Option<String>,
}

impl ProviderMetadata { #[must_use] pub fn into_json(self) -> serde_json::Value { serde_json::to_value(self).unwrap() } }

impl OidcServer {
	pub(crate) fn build(args: &crate::Args<'_>) -> Result<Self> {
		let db = Data {
			oidc_signingkey: args.db["oidc_signingkey"].clone(),
			oidcclientid_registration: args.db["oidcclientid_registration"].clone(),
			oidccode_authsession: args.db["oidccode_authsession"].clone(),
			oidcreqid_authrequest: args.db["oidcreqid_authrequest"].clone(),
		};

		let (signing_key_der, key_id) = match db.oidc_signingkey.get_blocking(SIGNING_KEY_DB_KEY).and_then(|handle| handle.deserialized::<Cbor<SigningKeyData>>().map(|cbor| cbor.0)) {
			| Ok(data) => { info!("Loaded existing OIDC signing key (kid={})", data.key_id); (data.key_der, data.key_id) },
			| Err(_) => {
				let (key_der, key_id) = Self::generate_signing_key()?;
				info!("Generated new OIDC signing key (kid={key_id})");
				let data = SigningKeyData { key_der: key_der.clone(), key_id: key_id.clone() };
				db.oidc_signingkey.raw_put(SIGNING_KEY_DB_KEY, Cbor(&data));
				(key_der, key_id)
			},
		};

		let jwk = Self::build_jwk(&signing_key_der, &key_id)?;
		Ok(Self { db, signing_key_der, jwk, key_id })
	}

	fn generate_signing_key() -> Result<(Vec<u8>, String)> {
		let rng = SystemRandom::new();
		let alg = &signature::ECDSA_P256_SHA256_FIXED_SIGNING;
		let pkcs8 = EcdsaKeyPair::generate_pkcs8(alg, &rng).map_err(|e| err!(error!("Failed to generate ECDSA key: {e}")))?;
		let key_id = utils::random_string(16);
		Ok((pkcs8.as_ref().to_vec(), key_id))
	}

	fn build_jwk(signing_key_der: &[u8], key_id: &str) -> Result<serde_json::Value> {
		let rng = SystemRandom::new();
		let alg = &signature::ECDSA_P256_SHA256_FIXED_SIGNING;
		let key_pair = EcdsaKeyPair::from_pkcs8(alg, signing_key_der, &rng).map_err(|e| err!(error!("Failed to load ECDSA key: {e}")))?;
		let public_bytes = key_pair.public_key().as_ref();
		let x = b64.encode(&public_bytes[1..33]);
		let y = b64.encode(&public_bytes[33..65]);
		Ok(serde_json::json!({"kty": "EC", "crv": "P-256", "use": "sig", "alg": "ES256", "kid": key_id, "x": x, "y": y}))
	}

	pub fn register_client(&self, request: DcrRequest) -> Result<OidcClientRegistration> {
		let client_id = utils::random_string(OIDC_CLIENT_ID_LENGTH);
		let auth_method = request.token_endpoint_auth_method.unwrap_or_else(|| "none".to_owned());
		let registration = OidcClientRegistration {
			client_id: client_id.clone(), redirect_uris: request.redirect_uris, client_name: request.client_name,
			client_uri: request.client_uri, logo_uri: request.logo_uri, contacts: request.contacts,
			token_endpoint_auth_method: auth_method,
			grant_types: request.grant_types.unwrap_or_else(|| vec!["authorization_code".to_owned(), "refresh_token".to_owned()]),
			response_types: request.response_types.unwrap_or_else(|| vec!["code".to_owned()]),
			application_type: request.application_type,
			registered_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs(),
		};
		self.db.oidcclientid_registration.raw_put(&*client_id, Cbor(&registration));
		Ok(registration)
	}

	pub async fn get_client(&self, client_id: &str) -> Result<OidcClientRegistration> {
		self.db.oidcclientid_registration.get(client_id).await.deserialized::<Cbor<_>>().map(|cbor: Cbor<OidcClientRegistration>| cbor.0).map_err(|_| err!(Request(NotFound("Unknown client_id"))))
	}

	pub async fn validate_redirect_uri(&self, client_id: &str, redirect_uri: &str) -> Result {
		let client = self.get_client(client_id).await?;
		if client.redirect_uris.iter().any(|uri| uri == redirect_uri) { Ok(()) } else { Err!(Request(InvalidParam("redirect_uri not registered for this client"))) }
	}

	pub fn store_auth_request(&self, req_id: &str, request: &OidcAuthRequest) { self.db.oidcreqid_authrequest.raw_put(req_id, Cbor(request)); }

	pub async fn take_auth_request(&self, req_id: &str) -> Result<OidcAuthRequest> {
		let request: OidcAuthRequest = self.db.oidcreqid_authrequest.get(req_id).await.deserialized::<Cbor<_>>().map(|cbor: Cbor<OidcAuthRequest>| cbor.0).map_err(|_| err!(Request(NotFound("Unknown or expired authorization request"))))?;
		self.db.oidcreqid_authrequest.remove(req_id);
		if SystemTime::now() > request.expires_at { return Err!(Request(NotFound("Authorization request has expired"))); }
		Ok(request)
	}

	#[must_use]
	pub fn create_auth_code(&self, auth_req: &OidcAuthRequest, user_id: OwnedUserId) -> String {
		let code = utils::random_string(AUTH_CODE_LENGTH);
		let now = SystemTime::now();
		let session = AuthCodeSession {
			code: code.clone(), client_id: auth_req.client_id.clone(), redirect_uri: auth_req.redirect_uri.clone(),
			scope: auth_req.scope.clone(), state: auth_req.state.clone(), nonce: auth_req.nonce.clone(),
			code_challenge: auth_req.code_challenge.clone(), code_challenge_method: auth_req.code_challenge_method.clone(),
			user_id, created_at: now, expires_at: now.checked_add(AUTH_CODE_LIFETIME).unwrap_or(now),
		};
		self.db.oidccode_authsession.raw_put(&*code, Cbor(&session));
		code
	}

	pub async fn exchange_auth_code(&self, code: &str, client_id: &str, redirect_uri: &str, code_verifier: Option<&str>) -> Result<AuthCodeSession> {
		let session: AuthCodeSession = self.db.oidccode_authsession.get(code).await.deserialized::<Cbor<_>>().map(|cbor: Cbor<AuthCodeSession>| cbor.0).map_err(|_| err!(Request(Forbidden("Invalid or expired authorization code"))))?;
		self.db.oidccode_authsession.remove(code);
		if SystemTime::now() > session.expires_at { return Err!(Request(Forbidden("Authorization code has expired"))); }
		if session.client_id != client_id { return Err!(Request(Forbidden("client_id mismatch"))); }
		if session.redirect_uri != redirect_uri { return Err!(Request(Forbidden("redirect_uri mismatch"))); }

		if let Some(challenge) = &session.code_challenge {
			let Some(verifier) = code_verifier else { return Err!(Request(Forbidden("code_verifier required for PKCE"))); };
			let method = session.code_challenge_method.as_deref().unwrap_or("S256");
			let computed = match method {
				| "S256" => { let hash = utils::hash::sha256::hash(verifier.as_bytes()); b64.encode(hash) },
				| "plain" => verifier.to_owned(),
				| _ => return Err!(Request(InvalidParam("Unsupported code_challenge_method"))),
			};
			if computed != *challenge { return Err!(Request(Forbidden("PKCE verification failed"))); }
		}

		Ok(session)
	}

	pub fn sign_id_token(&self, claims: &IdTokenClaims) -> Result<String> {
		let mut header = jwt::Header::new(jwt::Algorithm::ES256);
		header.kid = Some(self.key_id.clone());
		let key = jwt::EncodingKey::from_ec_der(&self.signing_key_der);
		jwt::encode(&header, claims, &key).map_err(|e| err!(error!("Failed to sign ID token: {e}")))
	}

	#[must_use] pub fn jwks(&self) -> serde_json::Value { serde_json::json!({"keys": [self.jwk.clone()]}) }

	#[must_use] pub fn at_hash(access_token: &str) -> String { let hash = utils::hash::sha256::hash(access_token.as_bytes()); b64.encode(&hash[..16]) }

	#[must_use] pub fn auth_request_lifetime() -> Duration { AUTH_REQUEST_LIFETIME }
}
