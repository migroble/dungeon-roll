extern crate proc_macro;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
use proc_macro::TokenStream;
use syn::{Data, DeriveInput};

#[proc_macro_derive(Dice)]
pub fn derive_dice(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident;
    let (ns, variants): (Vec<u64>, Vec<_>) = if let Data::Enum(de) = ast.data {
        de.variants
            .into_iter()
            .enumerate()
            .map(|(n, v)| (n as u64, v.ident))
            .collect::<Vec<_>>()
            .iter()
            .cloned()
            .unzip()
    } else {
        unimplemented!("Enums only >:(");
    };
    let count = variants.len() as u64;

    let expanded = quote! {
        impl Dice for #name {
            fn nth(n: u64) -> Self {
                assert!(n < #count);
                match n {
                    #(#ns => { Self::#variants }),*
                    _ => unreachable!(),
                }
            }

            fn faces() -> u64 {
                #count
            }
        }
    };

    TokenStream::from(expanded)
}
