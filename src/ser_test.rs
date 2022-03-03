use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, AttributeArgs, Ident, Item, Lit, Meta, MetaList, NestedMeta, Type};

pub fn ser_test(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as Item);
    let name = match &input {
        Item::Struct(item) => &item.ident,
        Item::Enum(item) => &item.ident,
        _ => panic!("expected struct or enum"),
    };

    // Parse arguments.
    let mut constr = Constr::Default;
    let mut test_ark = true;
    let mut test_serde = true;
    let mut types = Vec::new();
    for arg in args {
        match arg {
            // Path arguments (as in #[ser_test(arg)])
            NestedMeta::Meta(Meta::Path(path)) => match path.get_ident() {
                Some(id) if *id == "random" => {
                    constr = Constr::Random(id.clone());
                }

                Some(id) if *id == "arbitrary" => {
                    constr = Constr::Arbitrary;
                }

                _ => panic!("invalid argument {:?}", path),
            },

            // List arguments (as in #[ser_test(arg(val))])
            NestedMeta::Meta(Meta::List(MetaList { path, nested, .. })) => match path.get_ident() {
                Some(id) if *id == "random" => {
                    if nested.len() != 1 {
                        panic!("random attribute takes 1 argument");
                    }
                    match &nested[0] {
                        NestedMeta::Meta(Meta::Path(p)) => match p.get_ident() {
                            Some(id) => {
                                constr = Constr::Random(id.clone());
                            }
                            None => panic!("random argument must be an identifier"),
                        },
                        _ => panic!("random argument must be an identifier"),
                    }
                }

                Some(id) if *id == "constr" => {
                    if nested.len() != 1 {
                        panic!("constr attribute takes 1 argument");
                    }
                    match &nested[0] {
                        NestedMeta::Meta(Meta::Path(p)) => match p.get_ident() {
                            Some(id) => {
                                constr = Constr::Method(id.clone());
                            }
                            None => panic!("constr argument must be an identifier"),
                        },
                        _ => panic!("constr argument must be an identifier"),
                    }
                }

                Some(id) if *id == "ark" => {
                    if nested.len() != 1 {
                        panic!("ark attribute takes 1 argument");
                    }
                    match &nested[0] {
                        NestedMeta::Lit(Lit::Bool(b)) => {
                            test_ark = b.value;
                        }
                        _ => panic!("ark argument must be a boolean"),
                    }
                }

                Some(id) if *id == "serde" => {
                    if nested.len() != 1 {
                        panic!("serde attribute takes 1 argument");
                    }
                    match &nested[0] {
                        NestedMeta::Lit(Lit::Bool(b)) => {
                            test_serde = b.value;
                        }
                        _ => panic!("serde argument must be a boolean"),
                    }
                }

                Some(id) if *id == "types" => {
                    let params = nested.iter().map(parse_type).collect::<Vec<_>>();
                    types.push(quote!(<#name<#(#params),*>>));
                }

                _ => panic!("invalid attribute {:?}", path),
            },

            _ => panic!("invalid argument {:?}", arg),
        }
    }

    let mut output = quote! {
        #input
    };

    if types.is_empty() {
        // If no explicit type parameters were given for us to test with, assume the type under test
        // takes no type parameters.
        types.push(quote!(<#name>));
    }

    for (i, ty) in types.into_iter().enumerate() {
        let constr = match &constr {
            Constr::Default => quote! { #ty::default() },
            Constr::Arbitrary => quote! {
                {
                    use arbitrary::Unstructured;
                    use rand_chacha::{rand_core::{RngCore, SeedableRng}, ChaChaRng};
                    let mut rng = ChaChaRng::from_seed([42u8; 32]);
                    let mut data = vec![0u8; 2048];
                    rng.fill_bytes(&mut data);
                    Unstructured::new(&data).arbitrary::#ty().unwrap()
                }
            },
            Constr::Random(f) => quote! {
                {
                    use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
                    let mut rng = ChaChaRng::from_seed([42u8; 32]);
                    #ty::#f(&mut rng)
                }
            },
            Constr::Method(f) => quote! {
                #ty::#f()
            },
        };

        let serde_test = if test_serde {
            let test_name = Ident::new(
                &format!("ser_test_serde_round_trip_{}_{}", name, i),
                Span::mixed_site(),
            );
            quote! {
                #[cfg(test)]
                #[test]
                fn #test_name() {
                    let obj = #constr;
                    let buf = bincode::serialize(&obj).unwrap();
                    assert_eq!(obj, bincode::deserialize(&buf).unwrap());
                }
            }
        } else {
            quote! {}
        };

        let ark_test = if test_ark {
            let test_name = Ident::new(
                &format!("ser_test_ark_serialize_round_trip_{}_{}", name, i),
                Span::mixed_site(),
            );
            quote! {
                #[cfg(test)]
                #[test]
                fn #test_name() {
                    use ark_serialize::*;
                    let obj = #constr;
                    let mut buf = Vec::new();
                    CanonicalSerialize::serialize(&obj, &mut buf).unwrap();
                    assert_eq!(obj, CanonicalDeserialize::deserialize(buf.as_slice()).unwrap());
                }
            }
        } else {
            quote! {}
        };

        output = quote! {
            #output
            #serde_test
            #ark_test
        };
    }

    output.into()
}

enum Constr {
    Default,
    Random(Ident),
    Arbitrary,
    Method(Ident),
}

fn parse_type(m: &NestedMeta) -> Type {
    match m {
        NestedMeta::Lit(Lit::Str(s)) => syn::parse_str(&s.value()).unwrap(),
        NestedMeta::Meta(Meta::Path(p)) => syn::parse2(p.to_token_stream()).unwrap(),
        _ => {
            panic!("expected type");
        }
    }
}
