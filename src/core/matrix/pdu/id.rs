use super::{Count, RawId};

pub type ShortRoomId = ShortId;
pub type ShortEventId = ShortId;
pub type ShortStateKey = ShortId;
pub type ShortId = u64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Id {
	pub shortroomid: ShortRoomId,
	pub shorteventid: Count,
}

impl From<RawId> for Id {
	#[inline]
	fn from(raw: RawId) -> Self {
		Self {
			shortroomid: u64::from_be_bytes(raw.shortroomid()),
			shorteventid: Count::from_unsigned(u64::from_be_bytes(raw.shorteventid())),
		}
	}
}
