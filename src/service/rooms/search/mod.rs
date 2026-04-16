use std::sync::Arc;

use futures::{Stream, StreamExt};
use ruma::{RoomId, UserId, api::client::search::search_events::v3::Criteria};
use tuwunel_core::{
	PduCount, Result,
	arrayvec::ArrayVec,
	implement,
	matrix::event::{Event, Matches},
	trace,
	utils::{
		ArrayVecExt, IterStream, ReadyExt, set,
		stream::{TryIgnore, WidebandExt},
		string::is_cjk,
	},
};
use tuwunel_database::{Interfix, Map, keyval::Val};

use crate::rooms::{
	short::ShortRoomId,
	timeline::{PduId, RawPduId},
};

pub struct Service {
	db: Data,
	services: Arc<crate::services::OnceServices>,
}

struct Data {
	tokenids: Arc<Map>,
}

#[derive(Clone, Debug)]
pub struct RoomQuery<'a> {
	pub room_id: &'a RoomId,
	pub user_id: Option<&'a UserId>,
	pub criteria: &'a Criteria,
	pub limit: usize,
	pub skip: usize,
}

type TokenId = ArrayVec<u8, TOKEN_ID_MAX_LEN>;

const TOKEN_ID_MAX_LEN: usize =
	size_of::<ShortRoomId>() + WORD_MAX_LEN + 1 + size_of::<RawPduId>();
const WORD_MAX_LEN: usize = 50;

impl crate::Service for Service {
	fn build(args: &crate::Args<'_>) -> Result<Arc<Self>> {
		Ok(Arc::new(Self {
			db: Data { tokenids: args.db["tokenids"].clone() },
			services: args.services.clone(),
		}))
	}

	fn name(&self) -> &str { crate::service::make_name(std::module_path!()) }
}

#[implement(Service)]
pub fn index_pdu(&self, shortroomid: ShortRoomId, pdu_id: &RawPduId, message_body: &str) {
	let batch = tokenize(message_body)
		.map(|word| {
			let mut key = shortroomid.to_be_bytes().to_vec();
			key.extend_from_slice(word.as_bytes());
			key.push(0xFF);
			key.extend_from_slice(pdu_id.as_ref()); // TODO: currently we save the room id a second time here
			key
		})
		.collect::<Vec<_>>();

	self.db
		.tokenids
		.insert_batch(batch.iter().map(|k| (k.as_slice(), &[])));
}

#[implement(Service)]
pub fn deindex_pdu(&self, shortroomid: ShortRoomId, pdu_id: &RawPduId, message_body: &str) {
	let batch = tokenize(message_body).map(|word| {
		let mut key = shortroomid.to_be_bytes().to_vec();
		key.extend_from_slice(word.as_bytes());
		key.push(0xFF);
		key.extend_from_slice(pdu_id.as_ref()); // TODO: currently we save the room id a second time here
		key
	});

	for token in batch {
		self.db.tokenids.remove(&token);
	}
}

#[implement(Service)]
pub async fn search_pdus<'a>(
	&'a self,
	query: &'a RoomQuery<'a>,
) -> Result<(usize, impl Stream<Item = impl Event + use<>> + Send + '_)> {
	let pdu_ids: Vec<_> = self.search_pdu_ids(query).await?.collect().await;

	let filter = &query.criteria.filter;
	let count = pdu_ids.len();
	let pdus = pdu_ids
		.into_iter()
		.stream()
		.wide_filter_map(async |result_pdu_id: RawPduId| {
			self.services
				.timeline
				.get_pdu_from_id(&result_pdu_id)
				.await
				.ok()
		})
		.ready_filter(|pdu| !pdu.is_redacted())
		.ready_filter(move |pdu| filter.matches(pdu))
		.wide_filter_map(async |pdu| {
			self.services
				.state_accessor
				.user_can_see_event(query.user_id?, pdu.room_id(), pdu.event_id())
				.await
				.then_some(pdu)
		})
		.skip(query.skip)
		.take(query.limit);

	Ok((count, pdus))
}

// result is modeled as a stream such that callers don't have to be refactored
// though an additional async/wrap still exists for now
#[implement(Service)]
pub async fn search_pdu_ids(
	&self,
	query: &RoomQuery<'_>,
) -> Result<impl Stream<Item = RawPduId> + Send + '_ + use<'_>> {
	let shortroomid = self
		.services
		.short
		.get_shortroomid(query.room_id)
		.await?;

	let pdu_ids = self
		.search_pdu_ids_query_room(query, shortroomid)
		.await;

	let iters = pdu_ids.into_iter().map(IntoIterator::into_iter);

	Ok(set::intersection(iters).stream())
}

#[implement(Service)]
async fn search_pdu_ids_query_room(
	&self,
	query: &RoomQuery<'_>,
	shortroomid: ShortRoomId,
) -> Vec<Vec<RawPduId>> {
	tokenize(&query.criteria.search_term)
		.stream()
		.wide_then(async |word| {
			self.search_pdu_ids_query_words(shortroomid, &word)
				.collect::<Vec<_>>()
				.await
		})
		.collect::<Vec<_>>()
		.await
}

/// Iterate over PduId's containing a word
#[implement(Service)]
fn search_pdu_ids_query_words<'a>(
	&'a self,
	shortroomid: ShortRoomId,
	word: &'a str,
) -> impl Stream<Item = RawPduId> + Send + '_ {
	self.search_pdu_ids_query_word(shortroomid, word)
		.map(move |key| -> RawPduId {
			let key = &key[prefix_len(word)..];
			key.into()
		})
}

/// Iterate over raw database results for a word
#[implement(Service)]
fn search_pdu_ids_query_word(
	&self,
	shortroomid: ShortRoomId,
	word: &str,
) -> impl Stream<Item = Val<'_>> + Send + '_ + use<'_> {
	// rustc says const'ing this not yet stable
	let end_id: RawPduId = PduId { shortroomid, count: PduCount::max() }.into();

	// Newest pdus first
	let end = make_tokenid(shortroomid, word, &end_id);
	let prefix = make_prefix(shortroomid, word);
	self.db
		.tokenids
		.rev_raw_keys_from(&end)
		.ignore_err()
		.ready_take_while(move |key| key.starts_with(&prefix))
}

#[implement(Service)]
pub async fn delete_all_search_tokenids_for_room(&self, room_id: &RoomId) -> Result {
	let prefix = (room_id, Interfix);

	self.db
		.tokenids
		.keys_prefix_raw(&prefix)
		.ignore_err()
		.ready_for_each(|key| {
			trace!("Removing key: {key:?}");
			self.db.tokenids.remove(key);
		})
		.await;

	Ok(())
}

/// Splits a string into tokens used as keys in the search inverted index
///
/// This may be used to tokenize both message bodies (for indexing) or search
/// queries (for querying).
///
/// For CJK (Chinese, Japanese, Korean) text, uses character-based tokenization
/// to support languages that don't use spaces between words.
pub fn tokenize(body: &str) -> impl Iterator<Item = String> + Send + '_ {
	TokenIterator::new(body)
}

/// Iterator that handles both regular word-based tokenization and CJK character tokenization
struct TokenIterator<'a> {
	text: &'a str,
	position: usize,
}

impl<'a> TokenIterator<'a> {
	fn new(text: &'a str) -> Self {
		Self { text, position: 0 }
	}
}

impl Iterator for TokenIterator<'_> {
	type Item = String;

	#[allow(clippy::string_slice)]
	fn next(&mut self) -> Option<Self::Item> {
		let remaining = &self.text[self.position..];
		if remaining.is_empty() {
			return None;
		}

		// Skip non-alphanumeric characters
		let start = remaining
			.char_indices()
			.find(|(_, c)| c.is_alphanumeric())
			.map(|(i, _)| i)?;

		// start comes from char_indices(), so it's a valid UTF-8 boundary
		let remaining = &remaining[start..];
		let first_char = remaining.chars().next()?;

		if is_cjk(first_char) {
			// For CJK characters, tokenize character by character
			let char_len = first_char.len_utf8();
			self.position = self.position.saturating_add(start).saturating_add(char_len);
			
			// Return single CJK character as token
			let token = first_char.to_lowercase().to_string();
			if token.len() <= WORD_MAX_LEN {
				Some(token)
			} else {
				self.next()
			}
		} else {
			// For non-CJK text, use word-based tokenization
			let word_end = remaining
				.char_indices()
				.skip(1)
				.find(|(_, c)| !c.is_alphanumeric())
				.map(|(i, _)| i)
				.unwrap_or(remaining.len());

			// word_end comes from char_indices() or len(), so it's a valid UTF-8 boundary
			let word = &remaining[..word_end];
			self.position = self.position.saturating_add(start).saturating_add(word_end);

			let token = word.to_lowercase();
			if token.len() <= WORD_MAX_LEN {
				Some(token)
			} else {
				self.next()
			}
		}
	}
}

fn make_tokenid(shortroomid: ShortRoomId, word: &str, pdu_id: &RawPduId) -> TokenId {
	let mut key = make_prefix(shortroomid, word);
	key.extend_from_slice(pdu_id.as_ref());
	key
}

fn make_prefix(shortroomid: ShortRoomId, word: &str) -> TokenId {
	let mut key = TokenId::new();
	key.extend_from_slice(&shortroomid.to_be_bytes());
	key.extend_from_slice(word.as_bytes());
	key.push(tuwunel_database::SEP);
	key
}

fn prefix_len(word: &str) -> usize {
	size_of::<ShortRoomId>()
		.saturating_add(word.len())
		.saturating_add(1)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_tokenize_english() {
		let text = "Hello world! This is a test.";
		let tokens: Vec<String> = tokenize(text).collect();
		assert_eq!(tokens, vec!["hello", "world", "this", "is", "a", "test"]);
	}

	#[test]
	fn test_tokenize_chinese() {
		let text = "你好世界";
		let tokens: Vec<String> = tokenize(text).collect();
		// Each Chinese character should be a separate token
		assert_eq!(tokens, vec!["你", "好", "世", "界"]);
	}

	#[test]
	fn test_tokenize_mixed() {
		let text = "Hello 你好 world";
		let tokens: Vec<String> = tokenize(text).collect();
		assert_eq!(tokens, vec!["hello", "你", "好", "world"]);
	}

	#[test]
	fn test_tokenize_chinese_with_punctuation() {
		let text = "你好，世界！";
		let tokens: Vec<String> = tokenize(text).collect();
		assert_eq!(tokens, vec!["你", "好", "世", "界"]);
	}

	#[test]
	fn test_tokenize_japanese() {
		let text = "こんにちは世界";
		let tokens: Vec<String> = tokenize(text).collect();
		// Both Hiragana and Kanji should be tokenized character by character
		assert_eq!(tokens, vec!["こ", "ん", "に", "ち", "は", "世", "界"]);
	}

	#[test]
	fn test_tokenize_empty() {
		let text = "";
		let tokens: Vec<String> = tokenize(text).collect();
		assert!(tokens.is_empty());
	}

	#[test]
	fn test_tokenize_only_punctuation() {
		let text = "!@#$%^&*()";
		let tokens: Vec<String> = tokenize(text).collect();
		assert!(tokens.is_empty());
	}

	#[test]
	fn test_tokenize_max_length() {
		// Create a word that exceeds WORD_MAX_LEN
		let long_word = "a".repeat(WORD_MAX_LEN + 10);
		let tokens: Vec<String> = tokenize(&long_word).collect();
		// Long words should be filtered out
		assert!(tokens.is_empty());
	}
}
