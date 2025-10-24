// Copyright 2025 FastLabs Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::iter::IntoIterator;

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use syn::Attribute;
use syn::Data;
use syn::DataEnum;
use syn::DataStruct;
use syn::DeriveInput;
use syn::Error;
use syn::Expr;
use syn::Field;
use syn::Fields;
use syn::Ident;
use syn::Lit;
use syn::LitStr;
use syn::Member;
use syn::Meta;
use syn::MetaList;
use syn::Path;
use syn::Result;
use syn::Token;
use syn::Variant;
use syn::parse_macro_input;
use syn::parse_str;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Mut;

#[proc_macro_derive(Traversable, attributes(traverse))]
pub fn derive_traversable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand_with(input, |stream| impl_traversable(stream, false))
}

#[proc_macro_derive(TraversableMut, attributes(traverse))]
pub fn derive_traversable_mut(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand_with(input, |stream| impl_traversable(stream, true))
}

fn expand_with(
    input: proc_macro::TokenStream,
    handler: impl Fn(DeriveInput) -> Result<TokenStream>,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    handler(input)
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}

fn extract_meta(attrs: Vec<Attribute>, attr_name: &str) -> Result<Option<Meta>> {
    let macro_attrs = attrs
        .into_iter()
        .filter(|attr| attr.path().is_ident(attr_name))
        .collect::<Vec<Attribute>>();

    if let Some(second) = macro_attrs.get(2) {
        return Err(Error::new_spanned(second, "duplicate attribute"));
    }

    macro_attrs
        .first()
        .map(|attr| Ok(attr.meta.clone()))
        .transpose()
}

#[derive(Default)]
struct Params(HashMap<Path, Meta>);

impl Params {
    fn from_attrs(attrs: Vec<Attribute>, attr_name: &str) -> Result<Self> {
        Ok(extract_meta(attrs, attr_name)?
            .map(|meta| {
                if let Meta::List(meta_list) = meta {
                    Self::from_meta_list(meta_list)
                } else {
                    Err(Error::new_spanned(meta, "invalid attribute"))
                }
            })
            .transpose()?
            .unwrap_or_default())
    }

    fn from_meta_list(meta_list: MetaList) -> Result<Self> {
        let mut params = HashMap::new();
        let nested = meta_list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in nested {
            let path = meta.path();
            let entry = params.entry(path.clone());
            if matches!(entry, Entry::Occupied(_)) {
                return Err(Error::new_spanned(path, "duplicate parameter"));
            }
            entry.or_insert(meta);
        }
        Ok(Self(params))
    }

    fn validate(&self, allowed_params: &[&str]) -> Result<()> {
        for path in self.0.keys() {
            if !allowed_params
                .iter()
                .any(|allowed_param| path.is_ident(allowed_param))
            {
                return Err(Error::new_spanned(
                    path,
                    format!(
                        "unknown parameter, supported: {}",
                        allowed_params.join(", ")
                    ),
                ));
            }
        }
        Ok(())
    }

    fn param(&mut self, name: &str) -> Result<Option<Param>> {
        self.0
            .remove(&Ident::new(name, Span::call_site()).into())
            .map(Param::from_meta)
            .transpose()
    }
}

impl Iterator for Params {
    type Item = Result<Param>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .keys()
            .next()
            .cloned()
            .map(|path| Param::from_meta(self.0.remove(&path).unwrap()))
    }
}

enum Param {
    Unit(Span),
    StringLiteral(Span, LitStr),
    NestedParams(Span),
}

impl Param {
    fn from_meta(meta: Meta) -> Result<Self> {
        let span = meta.span();
        match meta {
            Meta::Path(_) => Ok(Param::Unit(span)),
            Meta::List(_) => Ok(Param::NestedParams(span)),
            Meta::NameValue(name_value) => {
                if let Expr::Lit(expr_lit) = &name_value.value {
                    if let Lit::Str(lit_str) = &expr_lit.lit {
                        Ok(Param::StringLiteral(span, lit_str.clone()))
                    } else {
                        Err(Error::new_spanned(name_value, "invalid parameter"))
                    }
                } else {
                    Err(Error::new_spanned(name_value, "invalid parameter"))
                }
            }
        }
    }

    fn span(&self) -> Span {
        match self {
            Self::Unit(span) | Self::StringLiteral(span, _) | Self::NestedParams(span) => *span,
        }
    }

    fn unit(self) -> Result<()> {
        if let Self::Unit(_) = self {
            Ok(())
        } else {
            Err(Error::new(self.span(), "invalid parameter"))
        }
    }

    fn string_literal(self) -> Result<LitStr> {
        if let Self::StringLiteral(_, lit_str) = self {
            Ok(lit_str)
        } else {
            Err(Error::new(self.span(), "invalid parameter"))
        }
    }
}

fn impl_traversable(input: DeriveInput, mutable: bool) -> Result<TokenStream> {
    let mut params = Params::from_attrs(input.attrs, "traverse")?;
    params.validate(&["skip"])?;

    let skip_visit_self = params
        .param("skip")?
        .map(Param::unit)
        .transpose()?
        .is_some();

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let visitor = Ident::new(
        if mutable { "VisitorMut" } else { "Visitor" },
        Span::call_site(),
    );

    let enter_method = Ident::new(
        if mutable { "enter_mut" } else { "enter" },
        Span::call_site(),
    );

    let leave_method = Ident::new(
        if mutable { "leave_mut" } else { "leave" },
        Span::call_site(),
    );

    let enter_self = if skip_visit_self {
        None
    } else {
        Some(quote! {
            ::visitor::#visitor::#enter_method(visitor, self);
        })
    };

    let leave_self = if skip_visit_self {
        None
    } else {
        Some(quote! {
            ::visitor::#visitor::#leave_method(visitor, self);
        })
    };

    let traverse_fields = match input.data {
        Data::Struct(struct_) => traverse_struct(struct_, mutable),
        Data::Enum(enum_) => traverse_enum(enum_, mutable),
        Data::Union(union_) => {
            return Err(Error::new_spanned(
                union_.union_token,
                "unions are not supported",
            ));
        }
    }?;

    let impl_trait = Ident::new(
        if mutable {
            "TraversableMut"
        } else {
            "Traversable"
        },
        Span::call_site(),
    );

    let method = Ident::new(
        if mutable { "traverse_mut" } else { "traverse" },
        Span::call_site(),
    );

    let mut_modifier = if mutable {
        Some(Mut(Span::call_site()))
    } else {
        None
    };

    Ok(quote! {
        impl #impl_generics ::visitor::#impl_trait for #name #ty_generics #where_clause {
            fn #method<V: ::visitor::#visitor>(& #mut_modifier self, visitor: &mut V) {
                #enter_self
                #traverse_fields
                #leave_self
            }
        }
    })
}

fn traverse_struct(s: DataStruct, mutable: bool) -> Result<TokenStream> {
    s.fields
        .into_iter()
        .enumerate()
        .map(|(index, field)| {
            let member = field.ident.as_ref().map_or_else(
                || Member::Unnamed(index.into()),
                |ident| Member::Named(ident.clone()),
            );
            let mut_modifier = if mutable {
                Some(Mut(Span::call_site()))
            } else {
                None
            };
            traverse_field(&quote! { & #mut_modifier self.#member }, field, mutable)
        })
        .collect()
}

fn traverse_enum(e: DataEnum, mutable: bool) -> Result<TokenStream> {
    let variants = e
        .variants
        .into_iter()
        .map(|x| traverse_variant(x, mutable))
        .collect::<Result<TokenStream>>()?;
    Ok(quote! {
        match self {
            #variants
            _ => {}
        }
    })
}

fn traverse_variant(v: Variant, mutable: bool) -> Result<TokenStream> {
    let mut params = Params::from_attrs(v.attrs, "traverse")?;
    params.validate(&["skip"])?;
    if params.param("skip")?.map(Param::unit).is_some() {
        return Ok(TokenStream::new());
    }
    let name = v.ident;
    let destructuring = destructure_fields(v.fields.clone())?;
    let fields = v
        .fields
        .into_iter()
        .enumerate()
        .map(|(index, field)| {
            traverse_field(
                &field
                    .ident
                    .clone()
                    .unwrap_or_else(|| Ident::new(&format!("i{}", index), Span::call_site()))
                    .to_token_stream(),
                field,
                mutable,
            )
        })
        .collect::<Result<TokenStream>>()?;
    Ok(quote! {
        Self::#name #destructuring => {
            #fields
        }
    })
}

fn destructure_fields(fields: Fields) -> Result<TokenStream> {
    Ok(match fields {
        Fields::Named(fields) => {
            let field_list = fields
                .named
                .into_iter()
                .map(|field| {
                    let mut params = Params::from_attrs(field.attrs, "traverse")?;
                    let field_name = field.ident.unwrap();
                    Ok(if params.param("skip")?.map(Param::unit).is_some() {
                        quote! { #field_name: _ }
                    } else {
                        field_name.into_token_stream()
                    })
                })
                .collect::<Result<Vec<TokenStream>>>()?;
            quote! {
                { #( #field_list ),* }
            }
        }
        Fields::Unnamed(fields) => {
            let field_list = fields
                .unnamed
                .into_iter()
                .enumerate()
                .map(|(index, field)| {
                    let mut params = Params::from_attrs(field.attrs, "traverse")?;
                    Ok(if params.param("skip")?.map(Param::unit).is_some() {
                        quote! { _ }
                    } else {
                        Ident::new(&format!("i{index}",), Span::call_site()).into_token_stream()
                    })
                })
                .collect::<Result<Vec<TokenStream>>>()?;
            quote! {
                ( #( #field_list ),* )
            }
        }
        Fields::Unit => TokenStream::new(),
    })
}

fn traverse_field(value: &TokenStream, field: Field, mutable: bool) -> Result<TokenStream> {
    let mut params = Params::from_attrs(field.attrs, "traverse")?;
    params.validate(&["skip", "with"])?;

    if params.param("skip")?.map(Param::unit).is_some() {
        return Ok(TokenStream::new());
    }

    let traverse_fn = params.param("with")?.map_or_else(
        || {
            parse_str(if mutable {
                "::visitor::TraversableMut::traverse_mut"
            } else {
                "::visitor::Traversable::traverse"
            })
        },
        |param| param.string_literal()?.parse::<Path>(),
    )?;

    Ok(quote! {
        #traverse_fn(#value, visitor);
    })
}
