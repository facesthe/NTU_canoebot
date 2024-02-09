mod enum_parent;
mod utils;

/// Automatically implement `EnumParent` for nested enums.
#[proc_macro_derive(DeriveEnumParent, attributes(enum_parent))]
pub fn enum_parent(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    enum_parent::derive(input)
}
