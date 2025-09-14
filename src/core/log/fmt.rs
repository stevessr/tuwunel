use std::fmt::Write;

use super::Level;
use crate::Result;

pub fn markdown<S>(out: &mut S, level: &Level, span: &str, msg: &str) -> Result
where
	S: Write + ?Sized,
{
	let level = level.as_str().to_uppercase();
	writeln!(out, "`{level:>5}` `{span:^12}` `{msg}`")?;

	Ok(())
}

pub fn markdown_table<S>(out: &mut S, level: &Level, span: &str, msg: &str) -> Result
where
	S: Write + ?Sized,
{
	let level = level.as_str().to_uppercase();
	writeln!(out, "| {level:>5} | {span:^12} | {msg} |")?;

	Ok(())
}

pub fn markdown_table_head<S>(out: &mut S) -> Result
where
	S: Write + ?Sized,
{
	write!(out, "| level | span | message |\n| ------: | :-----: | :------- |\n")?;

	Ok(())
}
