#![cfg(test)]

use ruma::{RoomId, UserId};
use tuwunel_database::{SEP, serialize_to_vec};

const ROOM: &str = "!room:example.com";
const USER: &str = "@user:example.com";
const THREAD_ROOT: &str = "$thread_root_event_id";
const COUNT: u64 = 42;

fn room() -> &'static RoomId { ROOM.try_into().unwrap() }
fn user() -> &'static UserId { USER.try_into().unwrap() }

fn legacy_key() -> Vec<u8> {
	serialize_to_vec((room(), COUNT, user())).expect("serialize legacy key")
}

fn new_key(kind: &str) -> Vec<u8> {
	serialize_to_vec((room(), COUNT, user(), kind)).expect("serialize new key")
}

#[test]
fn unthreaded_new_key_is_legacy_plus_separator() {
	let legacy = legacy_key();
	let new = new_key("");

	assert_eq!(&new[..legacy.len()], &*legacy);
	assert_eq!(new.len(), legacy.len() + 1);
	assert_eq!(*new.last().unwrap(), SEP);
}

#[test]
fn main_key_appends_kind_after_separator() {
	let legacy = legacy_key();
	let main = new_key("main");

	assert_eq!(&main[..legacy.len()], &*legacy);
	assert_eq!(main[legacy.len()], SEP);
	assert_eq!(&main[legacy.len() + 1..], b"main");
}

#[test]
fn thread_key_appends_root_after_separator() {
	let legacy = legacy_key();
	let thread = new_key(THREAD_ROOT);

	assert_eq!(&thread[..legacy.len()], &*legacy);
	assert_eq!(thread[legacy.len()], SEP);
	assert_eq!(&thread[legacy.len() + 1..], THREAD_ROOT.as_bytes());
}

/// Sweep filter behavior under each (stored_kind, sweep_kind) pairing.
/// Mirrors the `key.ends_with(suffix) || (legacy_match && key.ends_with(user))`
/// check in `data::readreceipt_update`.
#[test]
fn sweep_filter_matrix() {
	let user_bytes = user().as_bytes();
	let suffix = |kind: &str| serialize_to_vec((user(), kind)).expect("serialize suffix");

	let matches = |stored: &[u8], sweep_kind: &str| -> bool {
		let s = suffix(sweep_kind);
		let legacy_match = sweep_kind.is_empty();

		stored.ends_with(&s) || (legacy_match && stored.ends_with(user_bytes))
	};

	let legacy = legacy_key();
	let empty = new_key("");
	let main = new_key("main");
	let thread_a = new_key(THREAD_ROOT);
	let thread_b = new_key("$other_root");

	assert!(matches(&legacy, ""), "Unthreaded sweep catches legacy 3-tuple row");
	assert!(matches(&empty, ""), "Unthreaded sweep catches own-shape row");
	assert!(!matches(&main, ""), "Unthreaded sweep skips Main row");
	assert!(!matches(&thread_a, ""), "Unthreaded sweep skips Thread row");

	assert!(!matches(&legacy, "main"), "Main sweep does not touch legacy row");
	assert!(!matches(&empty, "main"), "Main sweep does not touch Unthreaded row");
	assert!(matches(&main, "main"), "Main sweep catches Main row");
	assert!(!matches(&thread_a, "main"), "Main sweep does not touch Thread row");

	assert!(!matches(&legacy, THREAD_ROOT), "Thread sweep does not touch legacy row");
	assert!(!matches(&empty, THREAD_ROOT), "Thread sweep does not touch Unthreaded row");
	assert!(!matches(&main, THREAD_ROOT), "Thread sweep does not touch Main row");
	assert!(matches(&thread_a, THREAD_ROOT), "Thread sweep catches matching root");
	assert!(!matches(&thread_b, THREAD_ROOT), "Thread sweep does not touch other root");
}

/// `user_id` ends with `:example.com`; the bare-user-id `ends_with` check
/// must not collide with kind tails. Encoded `"main"` and event-id roots
/// (`$...`) end in different bytes than a server-name suffix.
#[test]
fn legacy_match_does_not_collide_with_kind_tails() {
	let user_bytes = user().as_bytes();

	assert!(!new_key("main").ends_with(user_bytes));
	assert!(!new_key(THREAD_ROOT).ends_with(user_bytes));
	assert!(!new_key("").ends_with(user_bytes), "empty kind ends with SEP, not user_id");
	assert!(legacy_key().ends_with(user_bytes));
}
