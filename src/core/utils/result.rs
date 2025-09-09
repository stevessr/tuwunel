mod and_then_ref;
mod debug_inspect;
mod flat_ok;
mod log_debug_err;
mod log_err;
mod map_expect;
mod not_found;
mod unwrap_infallible;

pub use self::{
	and_then_ref::AndThenRef, debug_inspect::DebugInspect, flat_ok::FlatOk,
	log_debug_err::LogDebugErr, log_err::LogErr, map_expect::MapExpect, not_found::NotFound,
	unwrap_infallible::UnwrapInfallible,
};

pub type Result<T = (), E = crate::Error> = std::result::Result<T, E>;
