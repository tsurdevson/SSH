//! Support for deriving the `Decode` trait on structs.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn try_derive_decode(input: DeriveInput) -> syn::Result<TokenStream> {
    let data = match input.data {
        syn::Data::Struct(ref data) => data,
        _ => abort!(
            input.ident,
            "can't derive `Decode` on this type: only `struct` types are allowed",
        ),
    };

    if data.fields.is_empty() {
        abort!(
            input.ident,
            "can't derive `Decode` on a struct with no fields"
        );
    }

    let container_attributes = crate::attributes::ContainerAttributes::try_from(&input)?;
    let body = derive_for_fields(&container_attributes, &data.fields)?;
    let decode_error_type = container_attributes.decode_error_type;
    let ident = input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::ssh_encoding::Decode for #ident #type_generics #where_clause {
            type Error = #decode_error_type;

            fn decode(reader: &mut impl ::ssh_encoding::Reader) -> ::core::result::Result<Self, Self::Error> {
                #body
            }
        }
    })
}

fn derive_for_fields(
    container_attributes: &crate::attributes::ContainerAttributes,
    fields: &syn::Fields,
) -> syn::Result<TokenStream> {
    let mut field_decoders = Vec::with_capacity(fields.len());
    let mut is_tuple_struct = false;
    for field in fields {
        let ty = &field.ty;
        let attrs = crate::attributes::FieldAttributes::try_from(field)?;
        let decode_field = if attrs.length_prefixed {
            quote! { reader.read_prefixed(|reader| <#ty as ::ssh_encoding::Decode>::decode(reader))? }
        } else {
            quote! { <#ty as ::ssh_encoding::Decode>::decode(reader)? }
        };
        if let Some(ident) = &field.ident {
            field_decoders.push(quote! { #ident: #decode_field });
        } else {
            field_decoders.push(quote! { #decode_field });
            is_tuple_struct = true;
        }
    }
    let body = if is_tuple_struct {
        quote! { Self( #(#field_decoders),* ) }
    } else {
        quote! { Self{ #(#field_decoders),* } }
    };

    let decode_error_type = container_attributes.decode_error_type.clone();
    let body = if container_attributes.length_prefixed {
        quote! {
            reader.read_prefixed(|reader| {
                Ok::<_, #decode_error_type>(#body)
            })
        }
    } else {
        quote! { Ok(#body) }
    };

    Ok(body)
}
