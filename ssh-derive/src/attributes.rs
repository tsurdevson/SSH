use proc_macro2::TokenStream;
use quote::quote;

pub(crate) struct ContainerAttributes {
    pub(crate) decode_error_type: TokenStream,
    pub(crate) length_prefixed: bool,
}

impl TryFrom<&syn::DeriveInput> for ContainerAttributes {
    type Error = syn::Error;

    fn try_from(input: &syn::DeriveInput) -> Result<Self, Self::Error> {
        let mut decode_error_type = quote! {::ssh_encoding::Error};
        let mut length_prefixed = false;
        for attr in &input.attrs {
            if attr.path().is_ident("ssh") {
                attr.parse_nested_meta(|meta| {
                    // #[ssh(decode_error(E))]
                    if meta.path.is_ident("decode_error") {
                        let content;
                        syn::parenthesized!(content in meta.input);
                        decode_error_type = content.parse()?;
                    }
                    // #[ssh(length_prefixed)]
                    else if meta.path.is_ident("length_prefixed") {
                        length_prefixed = true;
                    } else {
                        return Err(syn::Error::new_spanned(meta.path, "unknown attribute"));
                    }
                    Ok(())
                })?;
            }
        }

        Ok(Self {
            decode_error_type,
            length_prefixed,
        })
    }
}

pub(crate) struct FieldAttributes {
    pub(crate) length_prefixed: bool,
}

impl TryFrom<&syn::Field> for FieldAttributes {
    type Error = syn::Error;

    fn try_from(field: &syn::Field) -> Result<Self, Self::Error> {
        let mut length_prefixed = false;
        for attr in &field.attrs {
            if attr.path().is_ident("ssh") {
                attr.parse_nested_meta(|meta| {
                    // #[ssh(length_prefixed)]
                    if meta.path.is_ident("length_prefixed") {
                        length_prefixed = true;
                    } else {
                        return Err(syn::Error::new_spanned(meta.path, "unknown attribute"));
                    }
                    Ok(())
                })?;
            }
        }

        Ok(Self { length_prefixed })
    }
}
