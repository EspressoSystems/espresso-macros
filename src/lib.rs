use proc_macro::TokenStream;

mod generic_tests;

#[proc_macro_attribute]
pub fn generic_tests(args: TokenStream, input: TokenStream) -> TokenStream {
    generic_tests::generic_tests(args, input)
}

mod ser_test;

#[proc_macro_attribute]
pub fn ser_test(args: TokenStream, input: TokenStream) -> TokenStream {
    ser_test::ser_test(args, input)
}
