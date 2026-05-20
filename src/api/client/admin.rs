use std::result::Result as StdResult;

use axum::extract::State;
use futures::future::join3;
use hmac::{Hmac, Mac};
use ruma::{
	OwnedUserId, UserId,
	api::client::admin::{
		get_nonce, is_user_locked, is_user_suspended, lock_user, register, suspend_user,
	},
};
use sha1::Sha1;
use tuwunel_core::{Err, Result, err};
use tuwunel_service::users::Register;

use crate::{ClientIp, Ruma};

type HmacSha1 = Hmac<Sha1>;

/// MSC4323: ordered access control for the four admin endpoints.
///
/// Spec mandates: authorization MUST be checked before account lookups
/// (anti-enumeration). The pure self-target check fails first; the three
/// IO checks (admin gate + existence + admin-target) run concurrently;
/// failures are then reported in spec-mandated priority.
async fn authorize(services: &crate::State, caller: &UserId, target: &UserId) -> Result {
	if caller == target {
		return Err!(Request(Forbidden("You cannot suspend or lock your own account")));
	}

	if !services.globals.user_is_local(target) {
		return Err!(Request(InvalidParam("User is not local to this server")));
	}

	let (caller_admin, target_active, target_admin) = join3(
		services.admin.user_is_admin(caller),
		services.users.is_active(target),
		services.admin.user_is_admin(target),
	)
	.await;

	if !caller_admin {
		return Err!(Request(Forbidden("Only server administrators can use this endpoint")));
	}

	if !target_active {
		return Err!(Request(NotFound("Unknown user")));
	}

	if target_admin {
		return Err!(Request(Forbidden(
			"You cannot suspend or lock another server administrator"
		)));
	}

	Ok(())
}

/// # `GET /_matrix/client/v1/admin/suspend/{userId}`
pub(crate) async fn is_user_suspended_route(
	State(services): State<crate::State>,
	body: Ruma<is_user_suspended::v1::Request>,
) -> Result<is_user_suspended::v1::Response> {
	let user_id = &body.user_id;

	authorize(&services, body.sender_user(), user_id).await?;

	Ok(is_user_suspended::v1::Response::new(services.users.is_suspended(user_id).await))
}

/// # `PUT /_matrix/client/v1/admin/suspend/{userId}`
pub(crate) async fn suspend_user_route(
	State(services): State<crate::State>,
	body: Ruma<suspend_user::v1::Request>,
) -> Result<suspend_user::v1::Response> {
	let sender_user = body.sender_user();
	let user_id = &body.user_id;

	authorize(&services, sender_user, user_id).await?;

	if services.users.is_suspended(user_id).await == body.suspended {
		return Ok(suspend_user::v1::Response::new(body.suspended));
	}

	let action = match body.suspended {
		| true => {
			services.users.set_suspended(user_id, sender_user);
			"suspended"
		},
		| false => {
			services.users.clear_suspended(user_id);
			"unsuspended"
		},
	};

	if services.server.config.admin_room_notices {
		services
			.admin
			.send_text(&format!("{user_id} has been {action} by {sender_user}."))
			.await;
	}

	Ok(suspend_user::v1::Response::new(body.suspended))
}

/// # `GET /_matrix/client/v1/admin/lock/{userId}`
pub(crate) async fn is_user_locked_route(
	State(services): State<crate::State>,
	body: Ruma<is_user_locked::v1::Request>,
) -> Result<is_user_locked::v1::Response> {
	let user_id = &body.user_id;

	authorize(&services, body.sender_user(), user_id).await?;

	Ok(is_user_locked::v1::Response::new(services.users.is_locked(user_id).await))
}

/// # `PUT /_matrix/client/v1/admin/lock/{userId}`
pub(crate) async fn lock_user_route(
	State(services): State<crate::State>,
	body: Ruma<lock_user::v1::Request>,
) -> Result<lock_user::v1::Response> {
	let sender_user = body.sender_user();
	let user_id = &body.user_id;

	authorize(&services, sender_user, user_id).await?;

	if services.users.is_locked(user_id).await == body.locked {
		return Ok(lock_user::v1::Response::new(body.locked));
	}

	let action = match body.locked {
		| true => {
			services.users.set_locked(user_id, sender_user);
			"locked"
		},
		| false => {
			services.users.clear_locked(user_id);
			"unlocked"
		},
	};

	if services.server.config.admin_room_notices {
		services
			.admin
			.send_text(&format!("{user_id} has been {action} by {sender_user}."))
			.await;
	}

	Ok(lock_user::v1::Response::new(body.locked))
}

/// # `GET /_synapse/admin/v1/register`
///
/// Issues a short-lived nonce. Returns 404 when the shared secret is not set.
pub(crate) async fn admin_register_nonce_route(
	State(services): State<crate::State>,
	_body: Ruma<get_nonce::v1::Request>,
) -> Result<get_nonce::v1::Response> {
	services
		.admin
		.register_is_enabled()
		.then(|| services.admin.issue_register_nonce())
		.map(get_nonce::v1::Response::new)
		.ok_or_else(|| err!(Request(NotFound("Shared-secret registration is not configured"))))
}

/// # `POST /_synapse/admin/v1/register`
///
/// Out-of-band account creation authenticated by HMAC over the homeserver's
/// registration shared secret. Bypasses UIAA. Mirrors Synapse's endpoint of
/// the same name.
pub(crate) async fn admin_register_route(
	State(services): State<crate::State>,
	ClientIp(client): ClientIp,
	body: Ruma<register::v1::Request>,
) -> Result<register::v1::Response> {
	let Some(shared_secret) = services.admin.register_shared_secret() else {
		return Err!(Request(NotFound("Shared-secret registration is not configured")));
	};

	if body.password.is_empty() {
		return Err!(Request(InvalidParam("Password must not be empty")));
	}

	if !services.admin.consume_register_nonce(&body.nonce) {
		return Err!(Request(Forbidden("Unrecognised or expired nonce")));
	}

	verify_mac(shared_secret, &body)
		.map_err(|()| err!(Request(Forbidden("HMAC check failed"))))?;

	let user_id = resolve_local_user_id(services, &body.username)?;

	if services.users.exists(&user_id).await {
		return Err!(Request(UserInUse("User ID is not available")));
	}

	services
		.users
		.full_register(Register {
			user_id: Some(&user_id),
			password: Some(&body.password),
			displayname: body.displayname.as_deref(),
			grant_first_user_admin: false,
			..Default::default()
		})
		.await?;

	if body.admin {
		services.admin.make_user_admin(&user_id).await?;
	}

	let (access_token, expires_in) = services.users.generate_access_token(false);

	let device_id = services
		.users
		.create_device(
			&user_id,
			None,
			(Some(&access_token), expires_in),
			None,
			None,
			Some(client),
		)
		.await?;

	Ok(register::v1::Response::new(
		user_id,
		access_token,
		services.globals.server_name().to_owned(),
		device_id,
	))
}

fn resolve_local_user_id(services: crate::State, username: &str) -> Result<OwnedUserId> {
	let server_name = services.globals.server_name();

	let user_id = match username.starts_with('@') {
		| true => UserId::parse(username),
		| false => UserId::parse_with_server_name(username, server_name),
	}
	.map_err(|_| err!(Request(InvalidParam("Invalid user id"))))?;

	if user_id.server_name() != server_name {
		return Err!(Request(InvalidParam("User is not local to this server")));
	}

	Ok(user_id)
}

fn verify_mac(secret: &str, req: &register::v1::Request) -> StdResult<(), ()> {
	let admin = if req.admin {
		b"admin".as_slice()
	} else {
		b"notadmin".as_slice()
	};

	let parts = [req.nonce.as_bytes(), req.username.as_bytes(), req.password.as_bytes(), admin]
		.into_iter()
		.chain(req.user_type.as_deref().map(str::as_bytes));

	let mut mac = HmacSha1::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key size");
	for (i, part) in parts.enumerate() {
		if i > 0 {
			mac.update(b"\0");
		}
		mac.update(part);
	}

	let expected = decode_hex(&req.mac).ok_or(())?;

	mac.verify_slice(&expected).map_err(|_| ())
}

fn decode_hex(s: &str) -> Option<Vec<u8>> {
	s.len().is_multiple_of(2).then_some(())?;

	s.as_bytes()
		.chunks_exact(2)
		.map(|c| Some((hex_nibble(c[0])? << 4) | hex_nibble(c[1])?))
		.collect()
}

const fn hex_nibble(b: u8) -> Option<u8> {
	match b {
		| b'0'..=b'9' => Some(b.wrapping_sub(b'0')),
		| b'a'..=b'f' => Some(b.wrapping_sub(b'a').wrapping_add(10)),
		| b'A'..=b'F' => Some(b.wrapping_sub(b'A').wrapping_add(10)),
		| _ => None,
	}
}

#[cfg(test)]
mod tests {
	use super::{decode_hex, register, verify_mac};

	const SECRET: &str = "shared-secret-12345";
	const NONCE: &str = "thenonce0123456789abcdef";
	const USER: &str = "alice";
	const PASS: &str = "p4ssw0rd!";

	// Reference vectors precomputed with Python's `hmac.new(SECRET,
	// digestmod=sha1)` over `NONCE \0 USER \0 PASS \0 (admin|notadmin) [\0
	// user_type]`.
	const MAC_NOTADMIN: &str = "44e0ec50d52aaa4029731dfcfe7e22123fa4c53e";
	const MAC_ADMIN: &str = "7774d6962a728b48ca7cd41e99a5149a93ff1ec5";
	const MAC_ADMIN_BOT: &str = "10f12d86c7210410ee777edadf655033b8f36008";

	fn req(admin: bool, user_type: Option<&str>, mac: &str) -> register::v1::Request {
		register::v1::Request {
			nonce: NONCE.into(),
			username: USER.into(),
			displayname: None,
			password: PASS.into(),
			admin,
			user_type: user_type.map(Into::into),
			mac: mac.into(),
		}
	}

	#[test]
	fn verify_mac_accepts_notadmin_reference_vector() {
		assert!(verify_mac(SECRET, &req(false, None, MAC_NOTADMIN)).is_ok());
	}

	#[test]
	fn verify_mac_accepts_admin_reference_vector() {
		assert!(verify_mac(SECRET, &req(true, None, MAC_ADMIN)).is_ok());
	}

	#[test]
	fn verify_mac_accepts_admin_with_user_type() {
		assert!(verify_mac(SECRET, &req(true, Some("bot"), MAC_ADMIN_BOT)).is_ok());
	}

	#[test]
	fn verify_mac_admin_flag_is_part_of_the_mac() {
		// the notadmin MAC must not validate when admin=true and vice versa.
		assert!(verify_mac(SECRET, &req(true, None, MAC_NOTADMIN)).is_err());
		assert!(verify_mac(SECRET, &req(false, None, MAC_ADMIN)).is_err());
	}

	#[test]
	fn verify_mac_user_type_is_part_of_the_mac() {
		// omitting user_type from the request must not validate the with-user_type MAC.
		assert!(verify_mac(SECRET, &req(true, None, MAC_ADMIN_BOT)).is_err());
	}

	#[test]
	fn verify_mac_rejects_wrong_secret() {
		assert!(verify_mac("other-secret", &req(false, None, MAC_NOTADMIN)).is_err());
	}

	#[test]
	fn verify_mac_rejects_malformed_hex() {
		assert!(verify_mac(SECRET, &req(false, None, "not-hex")).is_err());
		assert!(verify_mac(SECRET, &req(false, None, "abc")).is_err()); // odd length
	}

	#[test]
	fn verify_mac_accepts_uppercase_hex() {
		assert!(verify_mac(SECRET, &req(false, None, &MAC_NOTADMIN.to_uppercase())).is_ok());
	}

	#[test]
	fn decode_hex_roundtrips() {
		assert_eq!(decode_hex("00ff10ab"), Some(vec![0x00, 0xFF, 0x10, 0xAB]));
		assert_eq!(decode_hex(""), Some(vec![]));
		assert_eq!(decode_hex("0"), None);
		assert_eq!(decode_hex("0g"), None);
	}
}
