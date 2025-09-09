#![cfg(test)]

#[test]
fn common_prefix() {
	let input = ["conduwuit", "conduit", "construct"];
	let output = super::common_prefix(&input);
	assert_eq!(output, "con");
}

#[test]
fn common_prefix_empty() {
	let input = ["abcdefg", "hijklmn", "opqrstu"];
	let output = super::common_prefix(&input);
	assert_eq!(output, "");
}

#[test]
fn common_prefix_none() {
	let input = [];
	let output = super::common_prefix(&input);
	assert_eq!(output, "");
}

#[test]
fn unquote_infallible() {
	use super::Unquote;

	assert_eq!("\"foo\"".unquote_infallible(), "foo");
	assert_eq!("\"foo".unquote_infallible(), "\"foo");
	assert_eq!("foo".unquote_infallible(), "foo");
}
