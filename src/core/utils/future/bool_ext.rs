//! Extended external extensions to futures::FutureExt

use std::marker::Unpin;

use futures::{
	Future, FutureExt,
	future::{
		Either::{Left, Right},
		select_ok, try_join, try_join_all, try_join3, try_join4,
	},
};

use crate::utils::BoolExt as _;

pub trait BoolExt
where
	Self: Future<Output = bool> + Send,
{
	type Result;

	fn or<B>(self, b: B) -> impl Future<Output = bool> + Send
	where
		B: Future<Output = bool> + Send + Unpin,
		Self: Sized + Unpin;

	fn and<B>(self, b: B) -> impl Future<Output = bool> + Send
	where
		B: Future<Output = bool> + Send,
		Self: Sized;

	fn and2<B, C>(self, b: B, c: C) -> impl Future<Output = bool> + Send
	where
		B: Future<Output = bool> + Send,
		C: Future<Output = bool> + Send,
		Self: Sized;

	fn and3<B, C, D>(self, b: B, c: C, d: D) -> impl Future<Output = bool> + Send
	where
		B: Future<Output = bool> + Send,
		C: Future<Output = bool> + Send,
		D: Future<Output = bool> + Send,
		Self: Sized;
}

impl<Fut> BoolExt for Fut
where
	Fut: Future<Output = bool> + Send,
{
	type Result = crate::Result<(), ()>;

	fn or<B>(self, b: B) -> impl Future<Output = bool> + Send
	where
		B: Future<Output = bool> + Send + Unpin,
		Self: Sized + Unpin,
	{
		let test = |test: bool| test.ok_or(Self::Result::Err(()));

		select_ok([Left(self.map(test)), Right(b.map(test))]).map(|res| res.is_ok())
	}

	fn and<B>(self, b: B) -> impl Future<Output = bool> + Send
	where
		B: Future<Output = bool> + Send,
		Self: Sized,
	{
		let test = |test: bool| test.ok_or(Self::Result::Err(()));

		try_join(self.map(test), b.map(test)).map(|res| res.is_ok())
	}

	fn and2<B, C>(self, b: B, c: C) -> impl Future<Output = bool> + Send
	where
		B: Future<Output = bool> + Send,
		C: Future<Output = bool> + Send,
		Self: Sized,
	{
		let test = |test: bool| test.ok_or(Self::Result::Err(()));

		try_join3(self.map(test), b.map(test), c.map(test)).map(|res| res.is_ok())
	}

	fn and3<B, C, D>(self, b: B, c: C, d: D) -> impl Future<Output = bool> + Send
	where
		B: Future<Output = bool> + Send,
		C: Future<Output = bool> + Send,
		D: Future<Output = bool> + Send,
		Self: Sized,
	{
		let test = |test: bool| test.ok_or(Self::Result::Err(()));

		try_join4(self.map(test), b.map(test), c.map(test), d.map(test)).map(|res| res.is_ok())
	}
}

pub fn and<I, F>(args: I) -> impl Future<Output = bool> + Send
where
	I: Iterator<Item = F> + Send,
	F: Future<Output = bool> + Send,
{
	type Result = crate::Result<(), ()>;

	let args = args.map(|a| a.map(|a| a.ok_or(Result::Err(()))));

	try_join_all(args).map(|res| res.is_ok())
}

pub fn or<I, F>(args: I) -> impl Future<Output = bool> + Send
where
	I: Iterator<Item = F> + Send,
	F: Future<Output = bool> + Send + Unpin,
{
	type Result = crate::Result<(), ()>;

	let args = args.map(|a| a.map(|a| a.ok_or(Result::Err(()))));

	select_ok(args).map(|res| res.is_ok())
}
