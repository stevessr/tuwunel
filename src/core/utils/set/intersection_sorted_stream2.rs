use std::{
	cmp::Ordering::{Equal, Greater, Less},
	pin::Pin,
};

use futures::{
	Stream,
	task::{Context, Poll},
};
use pin_project_lite::pin_project;

use super::poll_head;
use crate::ready_some;

pin_project! {
	struct IntersectionSortedStream2<S, Item> {
		#[pin] a: S,
		#[pin] b: S,
		peeked_a: Option<Item>,
		peeked_b: Option<Item>,
	}
}

/// Intersection of sets
///
/// Outputs the set of elements common to both streams. Both streams must be
/// ascending under a total order; multiplicity is preserved (a value with n
/// copies in `a` and m copies in `b` appears `min(n, m)` times in the output).
pub fn intersection_sorted_stream2<S, Item>(a: S, b: S) -> impl Stream<Item = Item> + Send
where
	S: Stream<Item = Item> + Send,
	Item: Ord + Send + Sync,
{
	IntersectionSortedStream2 { a, b, peeked_a: None, peeked_b: None }
}

impl<S, Item> Stream for IntersectionSortedStream2<S, Item>
where
	S: Stream<Item = Item>,
	Item: Ord,
{
	type Item = Item;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let mut this = self.project();
		loop {
			let a = ready_some!(poll_head(this.a.as_mut(), this.peeked_a, cx));
			let b = ready_some!(poll_head(this.b.as_mut(), this.peeked_b, cx));
			match a.cmp(b) {
				| Greater => *this.peeked_b = None,
				| Less => *this.peeked_a = None,
				| Equal => {
					*this.peeked_b = None;
					return Poll::Ready(this.peeked_a.take());
				},
			}
		}
	}
}
