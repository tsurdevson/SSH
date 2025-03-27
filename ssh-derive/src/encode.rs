//! Support for deriving the `Encode` trait on structs.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn try_derive_encode(input: DeriveInput) -> syn::Result<TokenStream> {
    let data = match input.data {
        syn::Data::Struct(data) => data,
        _ => abort!(
            input.ident,
            "can't derive `Encode` on this type: only `struct` types are allowed",
        ),
    };

    if data.fields.is_empty() {
        abort!(
            input.ident,
            "can't derive `Encode` on a struct with no fields"
        );
    }

    let mut field_lengths = Vec::with_capacity(data.fields.len());
    let mut field_encoders = Vec::with_capacity(data.fields.len());

    for (i, field) in data.fields.into_iter().enumerate() {
        let field_ident = field.ident.map_or(
            {
                let i = syn::Index::from(i);
                quote! {self.#i}
            },
            |ident| quote! {self.#ident},
        );
        field_lengths.push(quote! { ::ssh_encoding::Encode::encoded_len(&#field_ident)? });
        field_encoders.push(quote! { ::ssh_encoding::Encode::encode(&#field_ident, writer)?; });
    }

    let ident = input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::ssh_encoding::Encode for #ident #type_generics #where_clause {
            fn encoded_len(&self) -> ::ssh_encoding::Result<usize> {
                use ::ssh_encoding::CheckedSum;

                [
                    #(#field_lengths),*
                ]
                .checked_sum()
            }

            fn encode(&self, writer: &mut impl ::ssh_encoding::Writer) -> ::ssh_encoding::Result<()> {
                #(#field_encoders)*
                Ok(())
            }
        }
    })
}
