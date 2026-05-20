use std::{
	cmp::{Eq, Ord},
	pin::Pin,
};

use futures::{
	Stream,
	task::{Context, Poll},
};
use pin_project_lite::pin_project;

use crate::{is_equal_to, is_less_than, ready_some};

/// Intersection of sets
///
/// Outputs the set of elements common to all input sets. Inputs do not have to
/// be sorted. If inputs are sorted a more optimized function is available in
/// this suite and should be used.
pub fn intersection<Item, Iter, Iters>(mut input: Iters) -> impl Iterator<Item = Item> + Send
where
	Iters: Iterator<Item = Iter> + Clone + Send,
	Iter: Iterator<Item = Item> + Send,
	Item: Eq,
{
	input.next().into_iter().flat_map(move |first| {
		let input = input.clone();
		first.filter(move |targ| {
			input
				.clone()
				.all(|mut other| other.any(is_equal_to!(*targ)))
		})
	})
}

/// Intersection of sets
///
/// Outputs the set of elements common to all input sets. Inputs must be sorted.
pub fn intersection_sorted<Item, Iter, Iters>(
	mut input: Iters,
) -> impl Iterator<Item = Item> + Send
where
	Iters: Iterator<Item = Iter> + Clone + Send,
	Iter: Iterator<Item = Item> + Send,
	Item: Eq + Ord,
{
	input.next().into_iter().flat_map(move |first| {
		let mut input = input.clone().collect::<Vec<_>>();
		first.filter(move |targ| {
			input.iter_mut().all(|it| {
				it.by_ref()
					.skip_while(is_less_than!(targ))
					.peekable()
					.peek()
					.is_some_and(is_equal_to!(targ))
			})
		})
	})
}

/// Intersection of sets
///
/// Outputs the set of elements common to both streams. Streams must be sorted.
pub fn intersection_sorted_stream2<S, Item>(a: S, b: S) -> impl Stream<Item = Item> + Send
where
	S: Stream<Item = Item> + Send,
	Item: Ord + Send + Sync,
{
	IntersectionSortedStream2 { a, b, peeked_a: None, peeked_b: None }
}

pin_project! {
	struct IntersectionSortedStream2<S, Item> {
		#[pin] a: S,
		#[pin] b: S,
		peeked_a: Option<Item>,
		peeked_b: Option<Item>,
	}
}

impl<S, Item> Stream for IntersectionSortedStream2<S, Item>
where
	S: Stream<Item = Item>,
	Item: Ord,
{
	type Item = Item;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		use std::cmp::Ordering::{Equal, Greater, Less};

		let mut this = self.project();
		loop {
			let a = ready_some!(poll_head(this.a.as_mut(), this.peeked_a, cx));
			let b = ready_some!(poll_head(this.b.as_mut(), this.peeked_b, cx));
			match a.cmp(b) {
				| Less => *this.peeked_a = None,
				| Greater => *this.peeked_b = None,
				| Equal => {
					*this.peeked_b = None;
					return Poll::Ready(this.peeked_a.take());
				},
			}
		}
	}
}

fn poll_head<'p, S, T>(
	stream: Pin<&mut S>,
	peeked: &'p mut Option<T>,
	cx: &mut Context<'_>,
) -> Poll<Option<&'p T>>
where
	S: Stream<Item = T>,
{
	if peeked.is_none() {
		*peeked = std::task::ready!(stream.poll_next(cx));
	}

	Poll::Ready(peeked.as_ref())
}

/// Difference of sets
///
/// Outputs the set of elements found in `a` which are not found in `b`. Streams
/// must be sorted.
pub fn difference_sorted_stream2<Item, A, B>(a: A, b: B) -> impl Stream<Item = Item> + Send
where
	A: Stream<Item = Item> + Send,
	B: Stream<Item = Item> + Send,
	Item: Ord + Send + Sync,
{
	DifferenceSortedStream2 { a, b, peeked_a: None, peeked_b: None }
}

pin_project! {
	struct DifferenceSortedStream2<A, B, Item> {
		#[pin] a: A,
		#[pin] b: B,
		peeked_a: Option<Item>,
		peeked_b: Option<Item>,
	}
}

impl<A, B, Item> Stream for DifferenceSortedStream2<A, B, Item>
where
	A: Stream<Item = Item>,
	B: Stream<Item = Item>,
	Item: Ord,
{
	type Item = Item;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		use std::cmp::Ordering::{Equal, Greater, Less};

		let mut this = self.project();
		loop {
			let a = ready_some!(poll_head(this.a.as_mut(), this.peeked_a, cx));
			let Some(b) = std::task::ready!(poll_head(this.b.as_mut(), this.peeked_b, cx)) else {
				return Poll::Ready(this.peeked_a.take());
			};
			match a.cmp(b) {
				| Less => return Poll::Ready(this.peeked_a.take()),
				| Greater => *this.peeked_b = None,
				| Equal => {
					*this.peeked_a = None;
					*this.peeked_b = None;
				},
			}
		}
	}
}
