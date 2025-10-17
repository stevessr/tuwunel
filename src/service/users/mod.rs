pub mod device;
mod keys;
mod ldap;
mod profile;

use std::sync::Arc;

use futures::{FutureExt, Stream, StreamExt, TryFutureExt, TryStreamExt, future::join3};
use ruma::{
	OwnedMxcUri, OwnedRoomId, OwnedUserId, UserId,
	api::client::filter::FilterDefinition,
	events::{
		GlobalAccountDataEventType,
		ignored_user_list::IgnoredUserListEvent,
		room::member::{MembershipState, RoomMemberEventContent},
	},
};
use tuwunel_core::{
	Err, Result, debug_warn, err, is_equal_to,
	pdu::PduBuilder,
	trace,
	utils::{self, IterStream, ReadyExt, TryFutureExtExt, stream::TryIgnore},
	warn,
};
use tuwunel_database::{Deserialized, Json, Map};

pub use self::keys::parse_master_key;

pub struct Service {
	services: Arc<crate::services::OnceServices>,
	db: Data,
}

struct Data {
	keychangeid_userid: Arc<Map>,
	keyid_key: Arc<Map>,
	onetimekeyid_onetimekeys: Arc<Map>,
	openidtoken_expiresatuserid: Arc<Map>,
	logintoken_expiresatuserid: Arc<Map>,
	todeviceid_events: Arc<Map>,
	token_userdeviceid: Arc<Map>,
	userdeviceid_metadata: Arc<Map>,
	userdeviceid_token: Arc<Map>,
	userdeviceid_refresh: Arc<Map>,
	userfilterid_filter: Arc<Map>,
	userid_avatarurl: Arc<Map>,
	userid_blurhash: Arc<Map>,
	userid_devicelistversion: Arc<Map>,
	userid_displayname: Arc<Map>,
	userid_lastonetimekeyupdate: Arc<Map>,
	userid_masterkeyid: Arc<Map>,
	userid_password: Arc<Map>,
	userid_origin: Arc<Map>,
	userid_selfsigningkeyid: Arc<Map>,
	userid_usersigningkeyid: Arc<Map>,
	useridprofilekey_value: Arc<Map>,
}

impl crate::Service for Service {
	fn build(args: &crate::Args<'_>) -> Result<Arc<Self>> {
		Ok(Arc::new(Self {
			services: args.services.clone(),
			db: Data {
				keychangeid_userid: args.db["keychangeid_userid"].clone(),
				keyid_key: args.db["keyid_key"].clone(),
				onetimekeyid_onetimekeys: args.db["onetimekeyid_onetimekeys"].clone(),
				openidtoken_expiresatuserid: args.db["openidtoken_expiresatuserid"].clone(),
				logintoken_expiresatuserid: args.db["logintoken_expiresatuserid"].clone(),
				todeviceid_events: args.db["todeviceid_events"].clone(),
				token_userdeviceid: args.db["token_userdeviceid"].clone(),
				userdeviceid_metadata: args.db["userdeviceid_metadata"].clone(),
				userdeviceid_token: args.db["userdeviceid_token"].clone(),
				userdeviceid_refresh: args.db["userdeviceid_refresh"].clone(),
				userfilterid_filter: args.db["userfilterid_filter"].clone(),
				userid_avatarurl: args.db["userid_avatarurl"].clone(),
				userid_blurhash: args.db["userid_blurhash"].clone(),
				userid_devicelistversion: args.db["userid_devicelistversion"].clone(),
				userid_displayname: args.db["userid_displayname"].clone(),
				userid_lastonetimekeyupdate: args.db["userid_lastonetimekeyupdate"].clone(),
				userid_masterkeyid: args.db["userid_masterkeyid"].clone(),
				userid_password: args.db["userid_password"].clone(),
				userid_origin: args.db["userid_origin"].clone(),
				userid_selfsigningkeyid: args.db["userid_selfsigningkeyid"].clone(),
				userid_usersigningkeyid: args.db["userid_usersigningkeyid"].clone(),
				useridprofilekey_value: args.db["useridprofilekey_value"].clone(),
			},
		}))
	}

	fn name(&self) -> &str { crate::service::make_name(std::module_path!()) }
}

impl Service {
	/// Returns true/false based on whether the recipient/receiving user has
	/// blocked the sender
	pub async fn user_is_ignored(&self, sender_user: &UserId, recipient_user: &UserId) -> bool {
		self.services
			.account_data
			.get_global(recipient_user, GlobalAccountDataEventType::IgnoredUserList)
			.await
			.is_ok_and(|ignored: IgnoredUserListEvent| {
				ignored
					.content
					.ignored_users
					.keys()
					.any(|blocked_user| blocked_user == sender_user)
			})
	}

	/// Check if a user is an admin
	#[inline]
	pub async fn is_admin(&self, user_id: &UserId) -> bool {
		self.services.admin.user_is_admin(user_id).await
	}

	/// Create a new user account on this homeserver.
	///
	/// User origin is by default "password" (meaning that it will login using
	/// its user_id/password). Users with other origins (currently only "ldap"
	/// is available) have special login processes.
	#[inline]
	pub async fn create(
		&self,
		user_id: &UserId,
		password: Option<&str>,
		origin: Option<&str>,
	) -> Result {
		origin.map_or_else(
			|| self.db.userid_origin.insert(user_id, "password"),
			|origin| self.db.userid_origin.insert(user_id, origin),
		);
		self.set_password(user_id, password).await
	}

	/// Deactivate account
	pub async fn deactivate_account(&self, user_id: &UserId) -> Result {
		// Remove all associated devices
		self.all_device_ids(user_id)
			.for_each(|device_id| self.remove_device(user_id, device_id))
			.await;

		// Set the password to "" to indicate a deactivated account. Hashes will never
		// result in an empty string, so the user will not be able to log in again.
		// Systems like changing the password without logging in should check if the
		// account is deactivated.
		self.set_password(user_id, None).await?;

		// TODO: Unhook 3PID
		Ok(())
	}

	/// Check if a user has an account on this homeserver.
	#[inline]
	pub async fn exists(&self, user_id: &UserId) -> bool {
		self.db.userid_password.get(user_id).await.is_ok()
	}

	/// Check if account is deactivated
	pub async fn is_deactivated(&self, user_id: &UserId) -> Result<bool> {
		self.db
			.userid_password
			.get(user_id)
			.map_ok(|val| val.is_empty())
			.map_err(|_| err!(Request(NotFound("User does not exist."))))
			.await
	}

	/// Check if account is active, infallible
	pub async fn is_active(&self, user_id: &UserId) -> bool {
		!self.is_deactivated(user_id).await.unwrap_or(true)
	}

	/// Check if account is active, infallible
	pub async fn is_active_local(&self, user_id: &UserId) -> bool {
		self.services.globals.user_is_local(user_id) && self.is_active(user_id).await
	}

	/// Returns the number of users registered on this server.
	#[inline]
	pub async fn count(&self) -> usize { self.db.userid_password.count().await }

	/// Returns an iterator over all users on this homeserver (offered for
	/// compatibility)
	#[allow(
		clippy::iter_without_into_iter,
		clippy::iter_not_returning_iterator
	)]
	pub fn iter(&self) -> impl Stream<Item = OwnedUserId> + Send + '_ {
		self.stream().map(ToOwned::to_owned)
	}

	/// Returns an iterator over all users on this homeserver.
	pub fn stream(&self) -> impl Stream<Item = &UserId> + Send {
		self.db.userid_password.keys().ignore_err()
	}

	/// Returns a list of local users as list of usernames.
	///
	/// A user account is considered `local` if the length of it's password is
	/// greater then zero.
	pub fn list_local_users(&self) -> impl Stream<Item = &UserId> + Send + '_ {
		self.db
			.userid_password
			.stream()
			.ignore_err()
			.ready_filter_map(|(u, p): (&UserId, &[u8])| (!p.is_empty()).then_some(u))
	}

	/// Returns the origin of the user (password/LDAP/...).
	pub async fn origin(&self, user_id: &UserId) -> Result<String> {
		self.db
			.userid_origin
			.get(user_id)
			.await
			.deserialized()
	}

	/// Returns the password hash for the given user.
	pub async fn password_hash(&self, user_id: &UserId) -> Result<String> {
		self.db
			.userid_password
			.get(user_id)
			.await
			.deserialized()
	}

	/// Hash and set the user's password to the Argon2 hash
	pub async fn set_password(&self, user_id: &UserId, password: Option<&str>) -> Result {
		// Cannot change the password of a LDAP user. There are two special cases :
		// - a `None` password can be used to deactivate a LDAP user
		// - a "*" password is used as the default password of an active LDAP user
		if cfg!(feature = "ldap")
			&& password.is_some()
			&& password != Some("*")
			&& self
				.db
				.userid_origin
				.get(user_id)
				.await
				.deserialized::<String>()
				.is_ok_and(is_equal_to!("ldap"))
		{
			return Err!(Request(InvalidParam("Cannot change password of a LDAP user")));
		}

		password
			.map(utils::hash::password)
			.transpose()
			.map_err(|e| {
				err!(Request(InvalidParam("Password does not meet the requirements: {e}")))
			})?
			.map_or_else(
				|| self.db.userid_password.insert(user_id, b""),
				|hash| self.db.userid_password.insert(user_id, hash),
			);

		Ok(())
	}

	/// Returns the displayname of a user on this homeserver.
	pub async fn displayname(&self, user_id: &UserId) -> Result<String> {
		self.db
			.userid_displayname
			.get(user_id)
			.await
			.deserialized()
	}

	/// Sets a new displayname or removes it if displayname is None. You still
	/// need to notify all rooms of this change.
	pub fn set_displayname(&self, user_id: &UserId, displayname: Option<String>) {
		if let Some(displayname) = displayname {
			self.db
				.userid_displayname
				.insert(user_id, displayname);
		} else {
			self.db.userid_displayname.remove(user_id);
		}
	}

	/// Get the `avatar_url` of a user.
	pub async fn avatar_url(&self, user_id: &UserId) -> Result<OwnedMxcUri> {
		self.db
			.userid_avatarurl
			.get(user_id)
			.await
			.deserialized()
	}

	/// Sets a new avatar_url or removes it if avatar_url is None.
	pub fn set_avatar_url(&self, user_id: &UserId, avatar_url: Option<OwnedMxcUri>) {
		match avatar_url {
			| Some(avatar_url) => {
				self.db
					.userid_avatarurl
					.insert(user_id, &avatar_url);
			},
			| _ => {
				self.db.userid_avatarurl.remove(user_id);
			},
		}
	}

	/// Get the blurhash of a user.
	pub async fn blurhash(&self, user_id: &UserId) -> Result<String> {
		self.db
			.userid_blurhash
			.get(user_id)
			.await
			.deserialized()
	}

	/// Sets a new avatar_url or removes it if avatar_url is None.
	pub fn set_blurhash(&self, user_id: &UserId, blurhash: Option<String>) {
		if let Some(blurhash) = blurhash {
			self.db.userid_blurhash.insert(user_id, blurhash);
		} else {
			self.db.userid_blurhash.remove(user_id);
		}
	}

	/// Creates a new sync filter. Returns the filter id.
	#[must_use]
	pub fn create_filter(&self, user_id: &UserId, filter: &FilterDefinition) -> String {
		let filter_id = utils::random_string(4);

		let key = (user_id, &filter_id);
		self.db.userfilterid_filter.put(key, Json(filter));

		filter_id
	}

	pub async fn get_filter(
		&self,
		user_id: &UserId,
		filter_id: &str,
	) -> Result<FilterDefinition> {
		let key = (user_id, filter_id);
		self.db
			.userfilterid_filter
			.qry(&key)
			.await
			.deserialized()
	}

	/// Creates an OpenID token, which can be used to prove that a user has
	/// access to an account (primarily for integrations)
	pub fn create_openid_token(&self, user_id: &UserId, token: &str) -> Result<u64> {
		use std::num::Saturating as Sat;

		let expires_in = self.services.server.config.openid_token_ttl;
		let expires_at = Sat(utils::millis_since_unix_epoch()) + Sat(expires_in) * Sat(1000);

		let mut value = expires_at.0.to_be_bytes().to_vec();
		value.extend_from_slice(user_id.as_bytes());

		self.db
			.openidtoken_expiresatuserid
			.insert(token.as_bytes(), value.as_slice());

		Ok(expires_in)
	}

	/// Find out which user an OpenID access token belongs to.
	pub async fn find_from_openid_token(&self, token: &str) -> Result<OwnedUserId> {
		let Ok(value) = self
			.db
			.openidtoken_expiresatuserid
			.get(token)
			.await
		else {
			return Err!(Request(Unauthorized("OpenID token is unrecognised")));
		};

		let (expires_at_bytes, user_bytes) = value.split_at(0_u64.to_be_bytes().len());
		let expires_at =
			u64::from_be_bytes(expires_at_bytes.try_into().map_err(|e| {
				err!(Database("expires_at in openid_userid is invalid u64. {e}"))
			})?);

		if expires_at < utils::millis_since_unix_epoch() {
			debug_warn!("OpenID token is expired, removing");
			self.db
				.openidtoken_expiresatuserid
				.remove(token.as_bytes());

			return Err!(Request(Unauthorized("OpenID token is expired")));
		}

		let user_string = utils::string_from_bytes(user_bytes)
			.map_err(|e| err!(Database("User ID in openid_userid is invalid unicode. {e}")))?;

		OwnedUserId::try_from(user_string)
			.map_err(|e| err!(Database("User ID in openid_userid is invalid. {e}")))
	}

	/// Creates a short-lived login token, which can be used to log in using the
	/// `m.login.token` mechanism.
	#[must_use]
	pub fn create_login_token(&self, user_id: &UserId, token: &str) -> u64 {
		use std::num::Saturating as Sat;

		let expires_in = self.services.server.config.login_token_ttl;
		let expires_at = Sat(utils::millis_since_unix_epoch()) + Sat(expires_in);

		let value = (expires_at.0, user_id);
		self.db
			.logintoken_expiresatuserid
			.raw_put(token, value);

		expires_in
	}

	/// Find out which user a login token belongs to.
	/// Removes the token to prevent double-use attacks.
	pub async fn find_from_login_token(&self, token: &str) -> Result<OwnedUserId> {
		let Ok(value) = self
			.db
			.logintoken_expiresatuserid
			.get(token)
			.await
		else {
			return Err!(Request(Forbidden("Login token is unrecognised")));
		};
		let (expires_at, user_id): (u64, OwnedUserId) = value.deserialized()?;

		if expires_at < utils::millis_since_unix_epoch() {
			trace!(?user_id, ?token, "Removing expired login token");

			self.db.logintoken_expiresatuserid.remove(token);

			return Err!(Request(Forbidden("Login token is expired")));
		}

		self.db.logintoken_expiresatuserid.remove(token);

		Ok(user_id)
	}

	#[cfg(not(feature = "ldap"))]
	pub async fn search_ldap(&self, _user_id: &UserId) -> Result<Vec<(String, bool)>> {
		Err!(FeatureDisabled("ldap"))
	}

	#[cfg(not(feature = "ldap"))]
	pub async fn auth_ldap(&self, _user_dn: &str, _password: &str) -> Result {
		Err!(FeatureDisabled("ldap"))
	}

	pub async fn update_displayname(
		&self,
		user_id: &UserId,
		displayname: Option<String>,
		rooms: &[OwnedRoomId],
	) {
		let (current_avatar_url, current_blurhash, current_displayname) = join3(
			self.services.users.avatar_url(user_id).ok(),
			self.services.users.blurhash(user_id).ok(),
			self.services.users.displayname(user_id).ok(),
		)
		.await;

		if displayname == current_displayname {
			return;
		}

		self.services
			.users
			.set_displayname(user_id, displayname.clone());

		// Send a new join membership event into rooms
		let avatar_url = &current_avatar_url;
		let blurhash = &current_blurhash;
		let displayname = &displayname;
		let rooms: Vec<_> = rooms
			.iter()
			.try_stream()
			.and_then(async |room_id: &OwnedRoomId| {
				let pdu = PduBuilder::state(user_id.to_string(), &RoomMemberEventContent {
					displayname: displayname.clone(),
					membership: MembershipState::Join,
					avatar_url: avatar_url.clone(),
					blurhash: blurhash.clone(),
					join_authorized_via_users_server: None,
					reason: None,
					is_direct: None,
					third_party_invite: None,
				});

				Ok((pdu, room_id))
			})
			.ignore_err()
			.collect()
			.await;

		self.update_all_rooms(user_id, rooms)
			.boxed()
			.await;
	}

	pub async fn update_avatar_url(
		&self,
		user_id: &UserId,
		avatar_url: Option<OwnedMxcUri>,
		blurhash: Option<String>,
		rooms: &[OwnedRoomId],
	) {
		let (current_avatar_url, current_blurhash, current_displayname) = join3(
			self.services.users.avatar_url(user_id).ok(),
			self.services.users.blurhash(user_id).ok(),
			self.services.users.displayname(user_id).ok(),
		)
		.await;

		if current_avatar_url == avatar_url && current_blurhash == blurhash {
			return;
		}

		self.services
			.users
			.set_avatar_url(user_id, avatar_url.clone());
		self.services
			.users
			.set_blurhash(user_id, blurhash.clone());

		// Send a new join membership event into rooms
		let avatar_url = &avatar_url;
		let blurhash = &blurhash;
		let displayname = &current_displayname;
		let rooms: Vec<_> = rooms
			.iter()
			.try_stream()
			.and_then(async |room_id: &OwnedRoomId| {
				let pdu = PduBuilder::state(user_id.to_string(), &RoomMemberEventContent {
					avatar_url: avatar_url.clone(),
					blurhash: blurhash.clone(),
					membership: MembershipState::Join,
					displayname: displayname.clone(),
					join_authorized_via_users_server: None,
					reason: None,
					is_direct: None,
					third_party_invite: None,
				});

				Ok((pdu, room_id))
			})
			.ignore_err()
			.collect()
			.await;

		self.update_all_rooms(user_id, rooms)
			.boxed()
			.await;
	}

	async fn update_all_rooms(&self, user_id: &UserId, rooms: Vec<(PduBuilder, &OwnedRoomId)>) {
		for (pdu_builder, room_id) in rooms {
			let state_lock = self.services.state.mutex.lock(room_id).await;
			if let Err(e) = self
				.services
				.timeline
				.build_and_append_pdu(pdu_builder, user_id, room_id, &state_lock)
				.await
			{
				warn!(%user_id, %room_id, "Failed to update/send new profile join membership update in room: {e}");
			}
		}
	}
}
