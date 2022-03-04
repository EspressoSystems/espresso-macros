// Copyright (c) 2022 Espresso Systems (espressosys.com)
// This file is part of the espresso-macros library.

// This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
// You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, parse_macro_input, Attribute, Item, ItemFn, ItemMod};

pub fn generic_tests(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut test_mod: ItemMod = parse_macro_input!(input);
    let name = &test_mod.ident;

    test_mod.content = test_mod.content.map(|(brace, items)| {
        // TODO A better way of declaring the instantiate macro would be to name it, simply,
        // `instantiate`, and always reference it by qualified name, e.g.
        // `tests::instantiate!(u32)`. This would eliminate the requirement of bringing both the
        // tests module and the macro into scope separately before invoking the macro. I believe
        // this is possible with the 2021 edition of Rust, but in the 2018 edition macros don't
        // behave like normal items of modules, and cannot be referenced by qualified path.
        let macro_name = format_ident!("instantiate_{}", name);
        let mut macro_body = proc_macro2::TokenStream::new();

        // Transform each item in the module by removing test attributes. For each test function
        // (function item which has at least one test attribute) append a monomorphized test
        // function to `macro_body`.
        let mut items = items
            .into_iter()
            .map(|item| {
                if let Item::Fn(mut f) = item {
                    let test_attrs = take_test_attrs(&mut f);
                    if !test_attrs.is_empty() {
                        let mut test_sig = f.sig.clone();
                        // The actual test function which gets defined by the macro must not have
                        // any generics.
                        test_sig.generics = Default::default();
                        let test_name = &test_sig.ident;
                        // The macro will take `$t:ty` as a parameter, so we can use `$t` to invoke
                        // the generic function with specific type parameters.
                        let basic_call = quote!(#name::#test_name::<$($t),*>());
                        // Async test functions require an `await`.
                        let call = if test_sig.asyncness.is_some() {
                            quote!(#basic_call.await)
                        } else {
                            basic_call
                        };
                        macro_body.extend(quote! {
                            #(#test_attrs)*
                            #test_sig {
                                #call
                            }
                        });
                    }
                    Item::Fn(f)
                } else {
                    item
                }
            })
            .collect::<Vec<_>>();

        items.push(
            parse2(quote! {
                #[macro_export]
                macro_rules! #macro_name {
                    ($($t:ty),*) => {
                        #macro_body
                    };
                }
            })
            .unwrap(),
        );

        (brace, items)
    });

    let output = quote! {
        #[macro_use]
        #test_mod
    };
    output.into()
}

fn take_test_attrs(f: &mut ItemFn) -> Vec<Attribute> {
    let (test_attrs, other_attrs) = std::mem::take(&mut f.attrs)
        .into_iter()
        .partition(is_test_attr);
    f.attrs = other_attrs;
    test_attrs
}

fn is_test_attr(attr: &Attribute) -> bool {
    matches!(
        attr.path
            .segments
            .last()
            .unwrap()
            .ident
            .to_string()
            .as_str(),
        "test" | "ignore" | "bench" | "should_panic"
    )
}
