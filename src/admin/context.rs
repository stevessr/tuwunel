use std::ops::Deref;

/// Orphan-rule wrapper: `#[implement(crate::Context)]` on each handler emits
/// an inherent impl, illegal on a foreign type.
pub(crate) struct Context<'a>(&'a tuwunel_service::admin::Context<'a>);

impl<'a> Context<'a> {
	#[inline]
	pub(crate) const fn new(inner: &'a tuwunel_service::admin::Context<'a>) -> Self {
		Self(inner)
	}
}

impl<'a> Deref for Context<'a> {
	type Target = tuwunel_service::admin::Context<'a>;

	#[inline]
	fn deref(&self) -> &Self::Target { self.0 }
}
