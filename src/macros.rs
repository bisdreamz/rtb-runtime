//! Shared macros for generating specification lists

/// Generate a specification list with constants and lookup functions.
///
/// This macro creates:
/// - Public constants for each value
/// - `name(value)` - Returns the constant name (e.g., "PHONE")
/// - `description(value)` - Returns the human-readable description (e.g., "Phone")
/// - `is_valid(value)` - Checks if a value is defined in this list
/// - `all_values()` - Returns all defined values as a slice
///
/// # Example
/// ```ignore
/// use rtb::spec_list;
///
/// spec_list! {
///     /// Phone device
///     PHONE = 4 => "Phone",
///
///     /// Tablet device
///     TABLET = 5 => "Tablet",
/// }
///
/// // Generated constants:
/// assert_eq!(PHONE, 4);
/// assert_eq!(TABLET, 5);
///
/// // Generated functions:
/// assert_eq!(name(4), Some("PHONE"));
/// assert_eq!(description(4), Some("Phone"));
/// assert!(is_valid(4));
/// assert_eq!(all_values(), &[4, 5]);
/// ```
#[macro_export]
macro_rules! spec_list {
    // Unsigned integers (u32) - default
    (
        $(
            $(#[$doc:meta])*
            $const_name:ident = $value:expr => $description:expr
        ),* $(,)?
    ) => {
        $(
            $(#[$doc])*
            pub const $const_name: u32 = $value;
        )*

        /// Get the constant name for a given value.
        pub fn name(value: u32) -> Option<&'static str> {
            match value {
                $($value => Some(stringify!($const_name)),)*
                _ => None,
            }
        }

        /// Get the description for a given value.
        pub fn description(value: u32) -> Option<&'static str> {
            match value {
                $($value => Some($description),)*
                _ => None,
            }
        }

        /// Check if a value is valid for this specification list.
        pub const fn is_valid(value: u32) -> bool {
            matches!(value, $($value)|*)
        }

        /// Get all valid values as a slice.
        pub const fn all_values() -> &'static [u32] {
            &[$($value),*]
        }
    };
}

/// Generate a specification list with signed integer constants (i32) and lookup functions.
#[macro_export]
macro_rules! spec_list_i32 {
    (
        $(
            $(#[$doc:meta])*
            $const_name:ident = $value:expr => $description:expr
        ),* $(,)?
    ) => {
        $(
            $(#[$doc])*
            pub const $const_name: i32 = $value;
        )*

        /// Get the constant name for a given value.
        pub fn name(value: i32) -> Option<&'static str> {
            match value {
                $($value => Some(stringify!($const_name)),)*
                _ => None,
            }
        }

        /// Get the description for a given value.
        pub fn description(value: i32) -> Option<&'static str> {
            match value {
                $($value => Some($description),)*
                _ => None,
            }
        }

        /// Check if a value is valid for this specification list.
        pub const fn is_valid(value: i32) -> bool {
            matches!(value, $($value)|*)
        }

        /// Get all valid values as a slice.
        pub const fn all_values() -> &'static [i32] {
            &[$($value),*]
        }
    };
}
