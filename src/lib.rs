use proc_macro::TokenStream;

mod generic_tests;

/// Define a set of generic tests which can be instantiated with different type parameters.
///
/// When applied to a module, `generic_tests` will transform the contents of that module as follows.
/// For each test function in the module (a test function is any function with an attribute ending
/// in `test` or `bench`, e.g. `#[test]` or `#[async_std::test]`), the test-relevant attributes
/// `#[test]`, `#[ignore]`, etc. will be removed. Otherwise, all items are left in the module
/// unchanged.
///
/// A macro will be added to the module which can be used to instantiate the generic tests in the
/// module. The name of the macro is `instantiate_` followed by the module name. Invoking the macro
/// with a list of type parameters in any context where the generic tests module is in scope is
/// equivalent to defining, for each test function in the module, a test function of the same name,
/// with the test-relevant attributes of the original generic function, whose body simply invokes
/// the generic function from the module with the given type parameters.
///
/// Note that, unlike normal test modules, all test functions must be public, since they will be
/// invoked from wherever the instantiate macro is invoked, which will be outside the module where
/// the tests are defined.
///
/// Also note that the instantiate macro is subject to the usual constraints on macro visibility. It
/// can only be used in a context that is lexically after the module where it is defined. To use it
/// from a module other than the one containing the generic tests module, all parent modules between
/// the generic tests module and the common ancestor of the invoking module must be annotated with
/// `#[macro_use]`. To use it from a different crate, `use my_crate::instantiate_macro` is required,
/// where `my_crate` is the external name of the crate containing the generic tests module.
///
/// # Example
/// ```
/// use zerok_macros::generic_tests;
///
/// #[generic_tests]
/// mod tests {
///     #[test]
///     pub fn a_test<T: std::fmt::Debug + Default + PartialEq>() {
///         assert_eq!(T::default(), T::default());
///     }
/// }
///
/// #[cfg(test)]
/// mod specific_tests {
///     use super::tests;
///     instantiate_tests!(u32);
///     instantiate_tests!(Vec<u32>);
/// }
/// ```
#[proc_macro_attribute]
pub fn generic_tests(args: TokenStream, input: TokenStream) -> TokenStream {
    generic_tests::generic_tests(args, input)
}

mod ser_test;

/// Generate round-trip serialization tests.
///
/// The attribute `ser_test` can be applied to a struct or enum definition to automatically derive
/// round-trip serialization tests for serde and ark_serialize impls. The generated tests follow the
/// pattern:
/// * Create an instance of the type under test, using a mechanism selected by the arguments to the
///   attribute macro (see Arguments below)
/// * Serialize the instance, checking that it succeeds
/// * Deserialize the serialized data and compare the result to the original instance
///
/// There are a few requirements on the type being tested:
/// * It must implement `Debug` and `PartialEq`
/// * It must implement `Default` unless a different construction method is used (see below)
/// * It must implement `Serialize` and `DeserializeOwned` unless `serde(false)` is used
/// * It must implement `CanonicalSerialize` and `CanonicalDeserialize` unless `ark(false)` is used
///
/// If testing a generic type, the `types(...)` attribute can be used to specify a comma-separated
/// list of type parameters to test with. Types must be enclosed in quotation marks if they are more
/// complex than just a path (e.g. if the type parameters themselves have type parameters). `types`
/// can be used more than once to test with different combinations of type parameters.
///
/// # Example
/// ```
/// use arbitrary::Arbitrary;
/// use ark_serialize::*;
/// use rand_chacha::ChaChaRng;
/// use serde::{Serialize, Deserialize};
/// use zerok_macros::ser_test;
///
/// // Deriving serde and ark_serialize tests using a default instance.
/// #[ser_test]
/// #[derive(
///     Debug, Default, PartialEq, Serialize, Deserialize, CanonicalSerialize, CanonicalDeserialize
/// )]
/// struct S1;
///
/// // Deriving serde tests only.
/// #[ser_test(ark(false))]
/// #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
/// struct S2;
///
/// // Deriving ark_serialize tests using an arbitrary instance.
/// #[ser_test(arbitrary, serde(false))]
/// #[derive(Arbitrary, Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
/// struct S3;
///
/// // Deriving tests using a random instance.
/// #[ser_test(random, ark(false))]
/// #[derive(Debug, PartialEq, Serialize, Deserialize)]
/// struct S4;
///
/// impl S4 {
///     fn random(rng: &mut ChaChaRng) -> Self {
///         S4
///     }
/// }
///
/// // Deriving tests using a random constructor with a non-standard name.
/// #[ser_test(random(random_for_test), ark(false))]
/// #[derive(Debug, PartialEq, Serialize, Deserialize)]
/// struct S5;
///
/// impl S5 {
///     #[cfg(test)]
///     fn random_for_test(rng: &mut ChaChaRng) -> Self {
///         S5
///     }
/// }
///
/// // Deriving tests using a custom constructor to get an instance.
/// #[ser_test(constr(new), ark(false))]
/// #[derive(Debug, PartialEq, Serialize, Deserialize)]
/// struct S6;
///
/// impl S6 {
///     fn new() -> Self {
///         S6
///     }
/// }
///
/// // Deriving tests for a generic type.
/// #[ser_test(types(u64, "Vec<u64>"), types(u32, bool), ark(false))]
/// #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
/// struct Generic<T1, T2> {
///     t1: T1,
///     t2: T2,
/// }
/// ```
///
/// # Arguments
/// * `ark([true|false])` opt in or out of `ark_serialize` tests (the default is `true`)
/// * `serde([true|false])` opt in or out of `serde` tests (the default is `true`)
/// * `arbitrary` use the type's `Arbitrary` implementation instead of `Default` to construct a test
///   instance
/// * `random` use the type's `random` associated function instead of `Default` to construct the
///   test instance. The `random` constructor must have a signature compatible with
///   `fn random(&mut ChaChaRng) -> Self`
/// * `random(f)` use the type's associated function `f` instead of `Default` to construct the test
///   instance. `f` mut have a signature compatible with `fn f(&mut ChaChaRng) -> Self`
/// * `constr(f)` use the type's associated function `f` instead of `Default` to construct the test
///   instance. `f` must have the signature `fn f() -> Self`
/// * `types(...)` test with the given type parameter list
#[proc_macro_attribute]
pub fn ser_test(args: TokenStream, input: TokenStream) -> TokenStream {
    ser_test::ser_test(args, input)
}
