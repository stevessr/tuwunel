mod bool_ext;
mod ext_ext;
mod option_ext;
mod option_stream;
mod ready_bool_ext;
mod ready_eq_ext;
mod try_ext_ext;

pub use self::{
	bool_ext::{BoolExt, and, and4, and5, and6, and7, or},
	ext_ext::ExtExt,
	option_ext::OptionFutureExt,
	option_stream::OptionStream,
	ready_bool_ext::ReadyBoolExt,
	ready_eq_ext::ReadyEqExt,
	try_ext_ext::TryExtExt,
};

#[macro_export]
macro_rules! ready_some {
	($e:expr) => {
		match $e {
			| std::task::Poll::Ready(Some(v)) => v,
			| std::task::Poll::Ready(None) => return std::task::Poll::Ready(None),
			| std::task::Poll::Pending => return std::task::Poll::Pending,
		}
	};
}
