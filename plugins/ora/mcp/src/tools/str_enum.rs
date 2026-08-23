//! Closed sets of string arguments.
//!
//! Declaring these as enums rather than `String` is what puts the valid values
//! into the JSON Schema the model sees, so a wrong value is unrepresentable
//! instead of merely discouraged by prose. The wire name is written once and
//! feeds both serde and the CLI argument, so the two cannot drift.

macro_rules! str_enum {
    (
        $(#[$meta:meta])*
        $name:ident { $($variant:ident => $wire:literal),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, rmcp::schemars::JsonSchema)]
        // Without this the value set lands in `$defs` behind a `$ref`, so the
        // model has to resolve an indirection to learn what it may pass.
        #[schemars(inline)]
        pub(crate) enum $name {
            $(#[serde(rename = $wire)] $variant),+
        }

        impl $name {
            pub(crate) const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $wire),+ }
            }

            #[cfg(test)]
            pub(crate) const ALL: &'static [Self] = &[$(Self::$variant),+];
        }
    };
}
