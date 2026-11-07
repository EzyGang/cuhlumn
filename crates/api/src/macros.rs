//! Shared implementations for validated string types.

/// Implements the string conversions of a validated string type.
///
/// The type must be a tuple struct over `String` with a `new` constructor that returns `Result<Self, $error>`.
macro_rules! string_type {
    ($name:ident, $error:ty) => {
        impl $name {
            /// Returns the value as a string slice.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = $error;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = $error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

/// Declares a fieldless enum that has a fixed text name for each variant.
///
/// Each name must be the lowercase form of its variant name, because serde and ts-rs use `rename_all = "lowercase"`.
/// The macro adds `ALL`, `as_str`, `Display`, `FromStr`, `TryFrom<String>`, and an error type that lists the valid names.
macro_rules! text_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident / $error:ident ($label:literal) {
            $($(#[$vmeta:meta])* $variant:ident = $text:literal,)+
        }
    ) => {
        $(#[$meta])*
        #[derive(
            Clone, Copy, Debug, Default, PartialEq, Eq, Hash,
            ::serde::Serialize, ::serde::Deserialize,
        )]
        #[cfg_attr(feature = "ts", derive(ts_rs::TS))]
        #[serde(rename_all = "lowercase")]
        #[cfg_attr(feature = "ts", ts(rename_all = "lowercase"))]
        $vis enum $name {
            $($(#[$vmeta])* $variant,)+
        }

        #[doc = concat!("A text that is not a valid ", $label, " name.")]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $error(String);

        impl ::std::error::Error for $error {}

        impl $name {
            /// Every variant, in declaration order.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// Returns the name used in JSON and in the database.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text,)+
                }
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = $error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($text => Ok(Self::$variant),)+
                    _ => Err($error(value.to_owned())),
                }
            }
        }

        impl TryFrom<String> for $name {
            type Error = $error;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                value.parse()
            }
        }

        impl ::std::fmt::Display for $error {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "the {} {:?} is not valid: use ", $label, self.0)?;
                crate::macros::write_names(f, $name::ALL.iter().map(|item| item.as_str()))
            }
        }
    };
}

/// Writes names as a list such as `a, b, or c`.
pub(crate) fn write_names<'a>(
    f: &mut ::std::fmt::Formatter<'_>,
    names: impl ExactSizeIterator<Item = &'a str>,
) -> ::std::fmt::Result {
    let count = names.len();
    for (index, name) in names.enumerate() {
        match index {
            0 => (),
            _ if index + 1 == count && count == 2 => f.write_str(" or ")?,
            _ if index + 1 == count => f.write_str(", or ")?,
            _ => f.write_str(", ")?,
        }
        f.write_str(name)?;
    }
    Ok(())
}
