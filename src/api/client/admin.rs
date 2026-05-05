use axum::extract::State;
use futures::future::join3;
use ruma::{
	UserId,
	api::client::admin::{is_user_locked, is_user_suspended, lock_user, suspend_user},
};
use tuwunel_core::{Err, Result};

use crate::Ruma;

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
