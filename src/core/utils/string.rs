mod between;
mod split;
mod tests;
mod unquote;
mod unquoted;

pub use self::{between::Between, split::SplitInfallible, unquote::Unquote, unquoted::Unquoted};
use crate::{Result, utils::exchange};

pub const EMPTY: &str = "";

/// Constant expression to bypass format! if the argument is a string literal
/// but not a format string. If the literal is a format string then String is
/// returned otherwise the input (i.e. &'static str) is returned. If multiple
/// arguments are provided the first is assumed to be a format string.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! format_maybe {
	($s:literal $(,)?) => {
		if $crate::is_format!($s) { std::format!($s).into() } else { $s.into() }
	};

	($s:literal, $($args:tt)+) => {
		std::format!($s, $($args)+).into()
	};
}

/// Constant expression to decide if a literal is a format string. Note: could
/// use some improvement.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! is_format {
	($s:literal) => {
		::const_str::contains!($s, "{") && ::const_str::contains!($s, "}")
	};

	($($s:tt)+) => {
		false
	};
}

#[inline]
pub fn collect_stream<F>(func: F) -> Result<String>
where
	F: FnOnce(&mut dyn std::fmt::Write) -> Result,
{
	let mut out = String::new();
	func(&mut out)?;
	Ok(out)
}

#[inline]
#[must_use]
pub fn camel_to_snake_string(s: &str) -> String {
	let est_len = s
		.chars()
		.fold(s.len(), |est, c| est.saturating_add(usize::from(c.is_ascii_uppercase())));

	let mut ret = String::with_capacity(est_len);
	camel_to_snake_case(&mut ret, s.as_bytes()).expect("string-to-string stream error");
	ret
}

#[inline]
#[allow(clippy::unbuffered_bytes)] // these are allocated string utilities, not file I/O utils
pub fn camel_to_snake_case<I, O>(output: &mut O, input: I) -> Result
where
	I: std::io::Read,
	O: std::fmt::Write,
{
	let mut state = false;
	input
		.bytes()
		.take_while(Result::is_ok)
		.map(Result::unwrap)
		.map(char::from)
		.try_for_each(|ch| {
			let m = ch.is_ascii_uppercase();
			let s = exchange(&mut state, !m);
			if m && s {
				output.write_char('_')?;
			}
			output.write_char(ch.to_ascii_lowercase())?;
			Result::<()>::Ok(())
		})
}

/// Find the common prefix from a collection of strings and return a slice
/// ```
/// use tuwunel_core::utils::string::common_prefix;
/// let input = ["conduwuit", "conduit", "construct"];
/// common_prefix(&input) == "con";
/// ```
#[must_use]
#[allow(clippy::string_slice)]
pub fn common_prefix<'a>(choice: &'a [&str]) -> &'a str {
	choice.first().map_or(EMPTY, move |best| {
		choice.iter().skip(1).fold(*best, |best, choice| {
			&best[0..choice
				.char_indices()
				.zip(best.char_indices())
				.take_while(|&(a, b)| a == b)
				.count()]
		})
	})
}

/// Parses the bytes into a string.
pub fn string_from_bytes(bytes: &[u8]) -> Result<String> {
	let str: &str = str_from_bytes(bytes)?;
	Ok(str.to_owned())
}

/// Parses the bytes into a string.
#[inline]
pub fn str_from_bytes(bytes: &[u8]) -> Result<&str> { Ok(std::str::from_utf8(bytes)?) }
