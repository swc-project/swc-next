//! Parse declaration metadata independently of Rust storage layout.
use syn::{Attribute, ItemEnum, LitStr, Meta, meta::ParseNestedMeta};

use crate::schema::{AstEnumVariant, AstFieldAttrs, EstreeAttrs, EstreeField, ScalarEnum};

fn string(meta: &ParseNestedMeta<'_>) -> syn::Result<String> {
    Ok(meta.value()?.parse::<LitStr>()?.value())
}

fn set_string(meta: &ParseNestedMeta<'_>, target: &mut Option<String>) -> syn::Result<()> {
    if target.is_some() {
        return Err(meta.error("duplicate ESTree attribute"));
    }
    *target = Some(string(meta)?);
    Ok(())
}

pub fn attrs(attributes: &[Attribute]) -> EstreeAttrs {
    let mut result = EstreeAttrs::default();
    for attr in attributes
        .iter()
        .filter(|attr| attr.path().is_ident("estree"))
    {
        // A standalone #[estree] marks scalar enums for declaration generation.
        if matches!(attr.meta, Meta::Path(_)) {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("fields") {
                return meta.parse_nested_meta(|field| {
                    let name = field
                        .path
                        .get_ident()
                        .ok_or_else(|| field.error("expected field name"))?
                        .to_string();
                    if result.fields.iter().any(|previous| previous.name == name) {
                        return Err(field.error("duplicate ESTree field"));
                    }
                    let mut ts_type = None;
                    let mut optional = false;
                    field.parse_nested_meta(|property| {
                        if property.path.is_ident("ts_type") {
                            set_string(&property, &mut ts_type)
                        } else if property.path.is_ident("optional") {
                            optional = true;
                            Ok(())
                        } else {
                            Err(property.error("unknown ESTree field attribute"))
                        }
                    })?;
                    result.fields.push(EstreeField {
                        name,
                        ts_type: ts_type.ok_or_else(|| field.error("expected ts_type"))?,
                        optional,
                    });
                    Ok(())
                });
            }
            let target = if meta.path.is_ident("rename") {
                &mut result.rename
            } else if meta.path.is_ident("ts_alias") {
                &mut result.ts_alias
            } else if meta.path.is_ident("ts_type") {
                &mut result.ts_type
            } else if meta.path.is_ident("add_ts_def") {
                &mut result.add_ts_def
            } else {
                return Err(meta.error("unknown ESTree type attribute"));
            };
            set_string(&meta, target)
        })
        .expect("invalid ESTree type attribute");
    }
    result
}

pub fn field_attrs(attributes: &[Attribute]) -> AstFieldAttrs {
    let mut result = AstFieldAttrs::default();
    for attr in attributes
        .iter()
        .filter(|attr| attr.path().is_ident("estree"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                result.skip = true;
                if meta.input.peek(syn::Token![=]) {
                    // Preserve the existing reason-valued codec skip while bare
                    // skip replaces the declaration-only flag.
                    let _ = string(&meta)?;
                    result.estree_skip = true;
                }
            } else if meta.path.is_ident("optional") {
                result.optional = true;
            } else if meta.path.is_ident("ts_type") {
                return set_string(&meta, &mut result.ts_type);
            } else if meta.path.is_ident("rename") {
                return set_string(&meta, &mut result.rename);
            } else {
                return Err(meta.error("unknown ESTree field attribute"));
            }
            Ok(())
        })
        .expect("invalid ESTree field attribute");
    }
    result
}

pub fn scalar(enumeration: &ItemEnum) -> Option<ScalarEnum> {
    if enumeration
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("ast"))
        || !enumeration
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("estree"))
    {
        return None;
    }
    let variants = enumeration
        .variants
        .iter()
        .map(|variant| {
            assert!(
                matches!(variant.fields, syn::Fields::Unit),
                "expected unit variant"
            );
            AstEnumVariant {
                type_id: None,
                name: variant.ident.to_string(),
                estree: attrs(&variant.attrs),
            }
        })
        .collect();
    Some(ScalarEnum {
        name: enumeration.ident.to_string(),
        attrs: attrs(&enumeration.attrs),
        variants,
    })
}
