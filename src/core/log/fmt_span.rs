use tracing_subscriber::fmt::format::FmtSpan;

#[inline]
#[must_use]
#[allow(clippy::match_same_arms)]
pub fn from_str(str: &str) -> FmtSpan {
	match str.to_uppercase().as_str() {
		| "ENTER" => FmtSpan::ENTER,
		| "EXIT" => FmtSpan::EXIT,
		| "NEW" => FmtSpan::NEW,
		| "CLOSE" => FmtSpan::CLOSE,
		| "ACTIVE" => FmtSpan::ACTIVE,
		| "FULL" => FmtSpan::FULL,
		| "NONE" => FmtSpan::NONE,
		| _ => FmtSpan::NONE,
	}
}
