use std::{collections::BTreeMap, time::Duration};

use ruma::{
	CanonicalJsonObject, CanonicalJsonValue, OwnedServerName, RoomId, RoomVersionId, ServerName,
	SigningKeyAlgorithm,
	api::{error::ErrorKind, federation::policy::sign_event::v1 as sign_event},
	events::{StateEventType, room::policy::RoomPolicyEventContent},
	serde::Base64,
	signatures::{to_canonical_json_string_for_signing, verify_canonical_json_bytes},
};
use serde_json::value::to_raw_value;
use tuwunel_core::{
	Err, Result, debug, err, implement,
	matrix::{Event, pdu::into_outgoing_federation, room_version},
	trace, warn,
};

/// Resolved policy-server configuration for a room.
pub struct PolicyConfig {
	pub via: OwnedServerName,
	pub ed25519_public_key: Base64,
}

/// Returns the room's policy server configuration when one is in effect:
/// `m.room.policy` (empty state key) is set, parses cleanly, lists at least
/// one ed25519 public key, and the `via` server has a joined user in the room.
/// Any failure returns `None`, signalling "no policy server configured" so the
/// caller skips the gate entirely.
#[implement(super::Service)]
pub async fn lookup_policy_server(&self, room_id: &RoomId) -> Option<PolicyConfig> {
	let content: RoomPolicyEventContent = self
		.services
		.state_accessor
		.room_state_get_content(room_id, &StateEventType::RoomPolicy, "")
		.await
		.ok()?;

	let public_key = content
		.public_keys
		.get(&SigningKeyAlgorithm::Ed25519)
		.cloned()?;

	self.services
		.state_cache
		.server_in_room(&content.via, room_id)
		.await
		.then(|| PolicyConfig {
			via: content.via,
			ed25519_public_key: public_key,
		})
}

/// MSC4284: ask the room's policy server to sign an outgoing event. The
/// signature is folded into `pdu_json["signatures"]` so it persists with the
/// event and federates transitively to other servers in the room. Returns
/// `Forbidden` when the policy server explicitly refuses; network errors and
/// timeouts fail open with a warn log.
#[implement(super::Service)]
#[tracing::instrument(name = "policy_sign", level = "debug", skip_all)]
pub async fn sign_outgoing_pdu<E>(
	&self,
	pdu_json: &mut CanonicalJsonObject,
	pdu: &E,
) -> Result<()>
where
	E: Event,
{
	if !self.services.server.config.enable_policy_servers {
		return Ok(());
	}

	if is_policy_state_event(pdu) {
		return Ok(());
	}

	let Ok(room_version) = self
		.services
		.state
		.get_room_version(pdu.room_id())
		.await
	else {
		return Ok(());
	};

	let Some(policy) = self.lookup_policy_server(pdu.room_id()).await else {
		trace!(room_id = %pdu.room_id(), "no policy server configured");
		return Ok(());
	};

	if let Some(signature) = self
		.fetch_policy_signature(&policy, pdu_json, &room_version)
		.await?
	{
		insert_policy_signature(pdu_json, &policy.via, &signature);
		debug!(via = %policy.via, event_id = %pdu.event_id(), "folded policy server signature");
	}

	Ok(())
}

/// Calls the policy server's `/sign` endpoint. Returns `Some(signature)` on
/// success, `None` when the call fails-open (network error or timeout), and
/// `Err(Forbidden)` when the policy server explicitly refuses to sign.
#[implement(super::Service)]
#[tracing::instrument(name = "policy_fetch", level = "debug", skip_all, fields(via = %policy.via))]
async fn fetch_policy_signature(
	&self,
	policy: &PolicyConfig,
	pdu_json: &CanonicalJsonObject,
	room_version: &RoomVersionId,
) -> Result<Option<String>> {
	let outgoing = into_outgoing_federation(pdu_json.clone(), room_version);
	let raw = to_raw_value(&outgoing)
		.map_err(|e| err!(Database("failed to serialize PDU for policy /sign: {e}")))?;

	let timeout = Duration::from_secs(
		self.services
			.server
			.config
			.policy_server_request_timeout,
	);

	let response = match tokio::time::timeout(
		timeout,
		self.services
			.federation
			.execute(&policy.via, sign_event::Request::new(raw)),
	)
	.await
	{
		| Ok(Ok(response)) => response,
		| Ok(Err(error)) if error.kind() == ErrorKind::Forbidden =>
			return Err!(Request(Forbidden("Event was rejected by the room's policy server."))),
		| Ok(Err(error)) => {
			warn!(via = %policy.via, %error, "policy server /sign failed; failing open");
			return Ok(None);
		},
		| Err(elapsed) => {
			warn!(via = %policy.via, %elapsed, "policy server /sign timed out; failing open");
			return Ok(None);
		},
	};

	// MSC4284 unstable: a 200 OK with no signature for `via` is also refusal.
	response
		.ed25519_signature(&policy.via)
		.map(ToOwned::to_owned)
		.map(Some)
		.ok_or_else(|| {
			err!(Request(Forbidden("Event was rejected by the room's policy server.")))
		})
}

/// Outcome of an inbound policy-server signature check.
#[derive(Debug)]
pub enum PolicyCheck {
	/// No policy server is configured for this room (or feature is off, or
	/// the event is the policy state event itself). The caller should not
	/// modify its soft-fail decision based on policy considerations.
	NotApplicable,

	/// Policy server signature is present and verifies cleanly.
	Pass,

	/// Policy server signature is absent. Per MSC4284, the homeserver SHOULD
	/// either fetch one from the policy server (Phase C) or soft-fail.
	Missing,

	/// Policy server signature is present but failed cryptographic
	/// verification. Soft-fail.
	Invalid,
}

/// MSC4284: verify the policy server signature on an inbound PDU. Returns
/// `NotApplicable` for rooms without a configured policy server (the gate is
/// skipped); `Pass` when the signature verifies; `Missing` when no signature
/// is present for the configured server; `Invalid` when the signature is
/// present but cryptographic verification fails.
#[implement(super::Service)]
#[tracing::instrument(name = "policy_verify", level = "debug", skip_all)]
pub async fn check_inbound_policy_signature<E>(
	&self,
	pdu_json: &CanonicalJsonObject,
	pdu: &E,
) -> PolicyCheck
where
	E: Event,
{
	if !self.services.server.config.enable_policy_servers {
		return PolicyCheck::NotApplicable;
	}

	if is_policy_state_event(pdu) {
		return PolicyCheck::NotApplicable;
	}

	let Some(policy) = self.lookup_policy_server(pdu.room_id()).await else {
		return PolicyCheck::NotApplicable;
	};

	let Ok(room_version) = self
		.services
		.state
		.get_room_version(pdu.room_id())
		.await
	else {
		return PolicyCheck::NotApplicable;
	};

	let Ok(rules) = room_version::rules(&room_version) else {
		return PolicyCheck::NotApplicable;
	};

	let Some(signature_b64) = extract_policy_signature(pdu_json, &policy.via) else {
		return PolicyCheck::Missing;
	};

	let Ok(signature) = Base64::<ruma::serde::base64::Standard>::parse(signature_b64) else {
		return PolicyCheck::Invalid;
	};

	let Ok(redacted) = ruma::canonical_json::redact(pdu_json.clone(), &rules.redaction, None)
	else {
		return PolicyCheck::Invalid;
	};

	let Ok(canonical) = to_canonical_json_string_for_signing(&redacted) else {
		return PolicyCheck::Invalid;
	};

	verify_canonical_json_bytes(
		&SigningKeyAlgorithm::Ed25519,
		policy.ed25519_public_key.as_bytes(),
		signature.as_bytes(),
		canonical.as_bytes(),
	)
	.map(|()| PolicyCheck::Pass)
	.unwrap_or_else(|error| {
		debug!(via = %policy.via, %error, "policy server signature failed verification");
		PolicyCheck::Invalid
	})
}

fn is_policy_state_event<E: Event>(pdu: &E) -> bool {
	pdu.kind().to_cow_str() == "m.room.policy" && pdu.state_key() == Some("")
}

fn extract_policy_signature<'a>(
	pdu_json: &'a CanonicalJsonObject,
	via: &ServerName,
) -> Option<&'a str> {
	let CanonicalJsonValue::Object(server_map) = pdu_json.get("signatures")? else {
		return None;
	};

	let CanonicalJsonValue::Object(key_map) = server_map.get(via.as_str())? else {
		return None;
	};

	let CanonicalJsonValue::String(signature) =
		key_map.get(RoomPolicyEventContent::POLICY_SERVER_ED25519_SIGNING_KEY_ID)?
	else {
		return None;
	};

	Some(signature.as_str())
}

fn insert_policy_signature(
	pdu_json: &mut CanonicalJsonObject,
	via: &ServerName,
	signature: &str,
) {
	let signatures = pdu_json
		.entry("signatures".into())
		.or_insert_with(|| CanonicalJsonValue::Object(BTreeMap::new()));

	let CanonicalJsonValue::Object(server_map) = signatures else {
		return;
	};

	let entry = server_map
		.entry(via.as_str().into())
		.or_insert_with(|| CanonicalJsonValue::Object(BTreeMap::new()));

	if let CanonicalJsonValue::Object(key_map) = entry {
		key_map.insert(
			RoomPolicyEventContent::POLICY_SERVER_ED25519_SIGNING_KEY_ID.into(),
			CanonicalJsonValue::String(signature.to_owned()),
		);
	}
}
