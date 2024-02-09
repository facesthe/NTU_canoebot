/// Provides a function that wraps the current callback value with respect to the main callback enum.
/// This trait is derived alongside [super::DeriveCallbackMenu].
/// ```no_run
/// #[derive(DeriveEnumParent)]
/// #[parent((_))] // if this is the root callback
/// pub enum RootCallback {
///     Here,
///     Inner(InnerCallback),
/// }
///
///
/// #[derive(DeriveEnumParent)]
/// #[parent(RootCallback::Inner(_))] // path from root callback to this type
/// pub enum InnerCallback {
///     Here
/// }
/// ```
pub trait EnumParent {
    type Parent;

    /// Wrap `Self` with the parent data type.
    fn enum_parent(value: Self) -> Self::Parent;
}
