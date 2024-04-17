//! A helper macro for creating structs with `new`.

#[doc(hidden)]
#[macro_export]
macro_rules! internal_new {
    ($struct:tt, $constructor:ident $($args:tt),*) => {
        <$struct>::$constructor($($args,)*)
    }
}

/// A helper for creating structs akin to the `new` keyword in other languages.
#[macro_export]
macro_rules! new {
    ($struct:tt $(,$args:tt)*) => {
        $crate::internal_new!($struct, new $($args),*)
    };
    ($struct:tt $constructor:tt: $($args:tt),*) => {
        $crate::internal_new!($struct, $constructor $($args),*)
    };
}

#[cfg(test)]
mod tests {
	use std::num::ParseIntError;

	#[derive(Debug, Default, PartialEq, Eq)]
	struct Empty(Option<String>);

	impl Empty {
		const fn new() -> Self {
			Self(None)
		}
	}

	#[derive(Debug, Default, Clone, PartialEq)]
	struct ManyArgs {
		value: u8,
		thing: Option<String>,
		other: bool,
		floating: f32,
	}

	impl ManyArgs {
		const fn new(value: u8, other: bool, floating: f32) -> Self {
			Self {
				value,
				thing: None,
				other,
				floating,
			}
		}

		fn with_thing(thing: String, value: u8, other: bool, floating: f32) -> Self {
			let mut this = Self::new(value, other, floating);
			this.thing.replace(thing);
			this
		}
	}

	#[derive(Debug, PartialEq, Eq)]
	#[repr(transparent)]
	struct TryNew(u8);

	impl TryNew {
		#[allow(clippy::self_named_constructors)]
		fn try_new(value: &str) -> Result<Self, ParseIntError> {
			Ok(Self(value.parse()?))
		}
	}

	#[derive(Debug, PartialEq, Eq)]
	#[repr(transparent)]
	struct TryOther(Option<TryNew>);

	impl TryOther {
		fn try_with_value(value: &str) -> Result<Self, ParseIntError> {
			let inner = TryNew::try_new(value)?;

			Ok(Self(Some(inner)))
		}
	}

	#[test]
	fn empty_constructor_works() {
		assert_eq!(new!(Empty), Empty::default());
	}

	#[test]
	fn constructor_with_args_works() {
		let value = new!(ManyArgs, 8u8, true, 7.0);
		assert_eq!(
			value,
			ManyArgs {
				value: 8,
				thing: None,
				other: true,
				floating: 7.0
			}
		);
	}

	#[test]
	fn other_constructor_works() {
		let thing = String::from("Hello, world!");
		assert_eq!(
			new!(ManyArgs with_thing: thing, 8u8, true, 7.0),
			ManyArgs {
				thing: Some("Hello, world!".to_owned()),
				value: 8,
				other: true,
				floating: 7.0,
			}
		);
	}

	#[test]
	fn try_constructors_work() -> Result<(), ParseIntError> {
		let try_new = new!(TryNew try_new: "7")?;

		assert_eq!(try_new, TryNew(7));

		let try_with_value = new!(TryOther try_with_value: "7")?;

		assert_eq!(try_with_value, TryOther(Some(TryNew(7))));

		Ok(())
	}
}
