//! A helper macro for creating structs with `new`.

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_new {
    ($struct:tt$(<$($gen:tt),*>)?, $constructor:ident $($args:tt),*) => {
        <$struct$(<$($gen)*,>)?>::$constructor($($args,)*)
    };
}

/// A helper for creating structs akin to the `new` keyword in other languages.
#[macro_export]
macro_rules! new {
    ($struct:tt$(<$($gen:tt),*>)?($($args:tt),*)) => {
        $crate::__internal_new!($struct$(<$($gen),*>)?, new $($args),*)
    };
    ($struct:tt$(<$($gen:tt),*>)?: $constructor:tt($($args:tt),*)) => {
        $crate::__internal_new!($struct$(<$($gen),*>)?, $constructor $($args),*)
    };
}

/// A shortcut for calling `try_*` constructors for structs.
#[macro_export]
macro_rules! try_new {
    ($struct:tt($($args:tt),*)) => {
        $crate::__internal_new!($struct, try_new $($args),*)
    };
    ($struct:tt$(<$($gen:tt),*>)?: $constructor:tt($($args:tt),*)) => {
        ::paste::paste! {
            $crate::__internal_new!($struct$(<$($gen),*>)?, [<try_ $constructor>] $($args),*)
        }
    };
}

/// A shortcut for calling `with_*` constructors for structs.
#[macro_export]
macro_rules! with {
    ($struct:tt$(<$($gen:tt),*>)?: $constructor:tt($($args:tt),*)) => {
        ::paste::paste! {
            $crate::__internal_new!($struct$(<$($gen),*>)?, [<with_ $constructor>] $($args),*)
        }
    }
}

/// A shortcut for calling `from_*`/[`from`] for structs.
///
/// [`from`]: std::convert::From::from
#[macro_export]
macro_rules! from {
	($struct:tt($($args:tt),*)) => {{
		use ::std::convert::From as _;
        $crate::__internal_new!($struct, from $($args),*)
	}};
    ($struct:tt$(<$($gen:tt),*>)?: $constructor:tt($($args:tt),*)) => {
        ::paste::paste! {
            $crate::__internal_new!($struct$(<$($gen),*>)?, [<from_ $constructor>] $($args),*)
        }
    }
}

/// A shortcut for calling `try_from_*`/[`try_from`] for structs.
///
/// [`try_from`]: std::convert::TryFrom::try_from
#[macro_export]
macro_rules! try_from {
    ($struct:tt($($args:tt),*)) => {{
        use ::std::convert::TryFrom as _;
        $crate::__internal_new!($struct, try_from $($args),*)
    }};
    ($struct:tt$(<$($gen:tt),*>)?: $constructor:tt($($args:tt),*)) => {
        ::paste::paste! {
            $crate::__internal_new!($struct$(<$($gen),*>)?, [<try_from_ $constructor>] $($args),*)
        }
    }
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

	impl<'a> TryFrom<&'a str> for TryNew {
		type Error = ParseIntError;

		fn try_from(value: &'a str) -> Result<Self, Self::Error> {
			Self::try_new(value)
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
		assert_eq!(new!(Empty()), Empty::default());
	}

	#[test]
	fn constructor_with_args_works() {
		let value = new!(ManyArgs(8u8, true, 7.0));
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
			new!(ManyArgs: with_thing(thing, 8u8, true, 7.0)),
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
		let try_new = try_new!(TryNew("7"))?;

		assert_eq!(try_new, TryNew(7));

		let try_with_value = try_new!(TryOther: with_value("7"))?;

		assert_eq!(try_with_value, TryOther(Some(TryNew(7))));

		Ok(())
	}

	#[test]
	fn constructor_with_generics_works() {
		assert_eq!(new!(Vec<u8>()), vec![]);
	}

	#[test]
	fn with_constructor_works() {
		let v = with!(Vec<u8>: capacity(7));

		assert_eq!(v.capacity(), 7);
	}

	#[test]
	fn convert_constructors_work() -> Result<(), ParseIntError> {
		let b = Box::new(5);

		let ptr = Box::into_raw(b);

		// SAFETY: It's the same pointer
		let new_b = unsafe { from!(Box<u8>: raw(ptr)) };

		assert_eq!(*new_b, 5);

		let value = try_from!(TryNew("7"))?;

		assert_eq!(value, TryNew(7));

		Ok(())
	}
}
