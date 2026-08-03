//! Implementation of `#[ach_module]`.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, Item, ItemMod, Lit, Meta};

/// Parsed attributes from `#[ach_module(name = "...")]`.
struct ModuleAttrs {
    name: String,
}

fn parse_attrs(attr: TokenStream) -> syn::Result<ModuleAttrs> {
    let meta_list: syn::punctuated::Punctuated<Meta, syn::Token![,]> = syn::parse::Parser::parse2(
        syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
        attr,
    )?;

    let mut name = None;

    for meta in &meta_list {
        if let Meta::NameValue(nv) = meta {
            if nv.path.is_ident("name") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(s), ..
                }) = &nv.value
                {
                    name = Some(s.value());
                }
            }
        }
    }

    let name = name.ok_or_else(|| {
        syn::Error::new(proc_macro2::Span::call_site(), "missing `name = \"...\"`")
    })?;

    Ok(ModuleAttrs { name })
}

/// Info extracted from each `#[ach_native(...)]` function in the module.
struct NativeInfo {
    fn_ident: syn::Ident,
    native_name: String,
    arity: i64,
    effects: String,
    capabilities: String,
    behavior: String,
    cancellation: String,
    resource: String,
    async_adapter: Option<String>,
}

/// Extract native metadata from a function's attributes.
fn extract_native_attr(attrs: &[syn::Attribute]) -> syn::Result<Option<NativeInfo>> {
    for attr in attrs {
        if attr.path().is_ident("ach_native") {
            let mut name = None;
            let mut arity = None;
            let mut effects = String::new();
            let mut capabilities = String::new();
            let mut behavior = "immediate".to_string();
            let mut cancellation = "none".to_string();
            let mut resource = "none".to_string();
            let mut async_adapter = None;

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let lit: Lit = value.parse()?;
                    if let Lit::Str(s) = lit {
                        name = Some(s.value());
                    }
                } else if meta.path.is_ident("arity") {
                    let value = meta.value()?;
                    // Handle negative arity (e.g., -1 for variadic)
                    let lookahead = value.lookahead1();
                    if lookahead.peek(syn::Token![-]) {
                        let _: syn::Token![-] = value.parse()?;
                        let lit: syn::LitInt = value.parse()?;
                        arity = Some(-(lit.base10_parse::<i64>()?));
                    } else {
                        let lit: syn::LitInt = value.parse()?;
                        arity = Some(lit.base10_parse::<i64>()?);
                    }
                } else if meta.path.is_ident("effects") {
                    effects = parse_string_value(&meta)?;
                } else if meta.path.is_ident("capabilities") {
                    capabilities = parse_string_value(&meta)?;
                } else if meta.path.is_ident("behavior") {
                    behavior = parse_string_value(&meta)?;
                } else if meta.path.is_ident("cancellation") {
                    cancellation = parse_string_value(&meta)?;
                } else if meta.path.is_ident("resource") {
                    resource = parse_string_value(&meta)?;
                } else if meta.path.is_ident("async_adapter") {
                    async_adapter = Some(parse_string_value(&meta)?);
                } else {
                    return Err(meta.error("unsupported ach_native metadata key"));
                }
                Ok(())
            })?;

            if let (Some(n), Some(a)) = (name, arity) {
                return Ok(Some(NativeInfo {
                    fn_ident: syn::Ident::new("placeholder", proc_macro2::Span::call_site()),
                    native_name: n,
                    arity: a,
                    effects,
                    capabilities,
                    behavior,
                    cancellation,
                    resource,
                    async_adapter,
                }));
            }
            return Err(syn::Error::new_spanned(
                attr,
                "ach_native requires name and arity",
            ));
        }
    }
    Ok(None)
}

fn parse_string_value(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<String> {
    let value = meta.value()?;
    let lit: Lit = value.parse()?;
    match lit {
        Lit::Str(value) => Ok(value.value()),
        _ => Err(meta.error("metadata value must be a string literal")),
    }
}

fn effect_tokens(spec: &str) -> syn::Result<TokenStream> {
    set_tokens(
        spec,
        "effect",
        quote! { ::akron::specs::EffectSet::empty() },
        |name| match name {
            "task" => Some(quote! { ::akron::specs::EffectSet::TASK }),
            "io.console" => Some(quote! { ::akron::specs::EffectSet::IO_CONSOLE }),
            "io.file" => Some(quote! { ::akron::specs::EffectSet::IO_FILE }),
            "io.network" => Some(quote! { ::akron::specs::EffectSet::IO_NETWORK }),
            "io.clock" => Some(quote! { ::akron::specs::EffectSet::IO_CLOCK }),
            "io.random" => Some(quote! { ::akron::specs::EffectSet::IO_RANDOM }),
            "prove" => Some(quote! { ::akron::specs::EffectSet::PROVE }),
            "verify" => Some(quote! { ::akron::specs::EffectSet::VERIFY }),
            "circuit" => Some(quote! { ::akron::specs::EffectSet::CIRCUIT }),
            "host.unknown" => Some(quote! { ::akron::specs::EffectSet::UNKNOWN_HOST }),
            _ => None,
        },
    )
}

fn capability_tokens(spec: &str) -> syn::Result<TokenStream> {
    set_tokens(
        spec,
        "capability",
        quote! { ::akron::specs::CapabilitySet::empty() },
        |name| match name {
            "console.read" => Some(quote! { ::akron::specs::CapabilitySet::CONSOLE_READ }),
            "console.write" => Some(quote! { ::akron::specs::CapabilitySet::CONSOLE_WRITE }),
            "file.read" => Some(quote! { ::akron::specs::CapabilitySet::FILE_READ }),
            "file.write" => Some(quote! { ::akron::specs::CapabilitySet::FILE_WRITE }),
            "network.connect" => Some(quote! { ::akron::specs::CapabilitySet::NETWORK_CONNECT }),
            "network.listen" => Some(quote! { ::akron::specs::CapabilitySet::NETWORK_LISTEN }),
            "clock" => Some(quote! { ::akron::specs::CapabilitySet::CLOCK }),
            "random" => Some(quote! { ::akron::specs::CapabilitySet::RANDOM }),
            "host.unknown" => Some(quote! { ::akron::specs::CapabilitySet::UNKNOWN_HOST }),
            _ => None,
        },
    )
}

fn set_tokens<F>(
    spec: &str,
    kind: &str,
    empty: TokenStream,
    mut lookup: F,
) -> syn::Result<TokenStream>
where
    F: FnMut(&str) -> Option<TokenStream>,
{
    let mut output = empty;
    for name in spec.split('|').filter(|name| !name.is_empty()) {
        let token = lookup(name).ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("unknown {kind} `{name}`"),
            )
        })?;
        output = quote! { #output | #token };
    }
    Ok(output)
}

fn behavior_tokens(value: &str) -> syn::Result<TokenStream> {
    match value {
        "immediate" => Ok(quote! { ::akron::specs::NativeBehavior::Immediate }),
        "blocking" => Ok(quote! { ::akron::specs::NativeBehavior::Blocking }),
        "suspending" => Ok(quote! { ::akron::specs::NativeBehavior::Suspending }),
        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("unknown native behavior `{value}`"),
        )),
    }
}

fn cancellation_tokens(value: &str) -> syn::Result<TokenStream> {
    match value {
        "none" => Ok(quote! { ::akron::specs::CancellationPolicy::None }),
        "before-start" => Ok(quote! { ::akron::specs::CancellationPolicy::BeforeStart }),
        "cooperative" => Ok(quote! { ::akron::specs::CancellationPolicy::Cooperative }),
        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("unknown cancellation policy `{value}`"),
        )),
    }
}

fn resource_tokens(value: &str) -> syn::Result<TokenStream> {
    if let Some((creates, borrows)) = value.split_once("+borrows:") {
        let created = creates.strip_prefix("creates:").ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("unknown resource effect `{value}`"),
            )
        })?;
        let created = resource_kind_tokens(created)?;
        let borrowed = resource_kind_tokens(borrows)?;
        return Ok(quote! {
            ::akron::specs::ResourceEffect::CreatesAndBorrows {
                created: #created,
                borrowed: #borrowed,
            }
        });
    }
    let (mode, kind) = match value.split_once(':') {
        Some(parts) => parts,
        None if value == "none" => {
            return Ok(quote! { ::akron::specs::ResourceEffect::None });
        }
        None => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("unknown resource effect `{value}`"),
            ));
        }
    };
    let kind = resource_kind_tokens(kind)?;
    match mode {
        "creates" => Ok(quote! { ::akron::specs::ResourceEffect::Creates(#kind) }),
        "consumes" => Ok(quote! { ::akron::specs::ResourceEffect::Consumes(#kind) }),
        "borrows" => Ok(quote! { ::akron::specs::ResourceEffect::Borrows(#kind) }),
        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("unknown resource mode `{mode}`"),
        )),
    }
}

fn resource_kind_tokens(kind: &str) -> syn::Result<TokenStream> {
    Ok(match kind {
        "file" => quote! { ::akron::specs::ResourceKind::File },
        "listener" => quote! { ::akron::specs::ResourceKind::Listener },
        "connection" => quote! { ::akron::specs::ResourceKind::Connection },
        "channel" => quote! { ::akron::specs::ResourceKind::Channel },
        "task" => quote! { ::akron::specs::ResourceKind::Task },
        _ => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("unknown resource kind `{kind}`"),
            ));
        }
    })
}

pub fn ach_module_impl(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let attrs = parse_attrs(attr)?;
    let module: ItemMod = parse2(item)?;

    let mod_ident = &module.ident;
    let mod_vis = &module.vis;

    // Generate struct name: "string_ext" → StringExtModule
    let struct_name = {
        let pascal: String = attrs
            .name
            .split('_')
            .map(|part| {
                let mut c = part.chars();
                match c.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                    None => String::new(),
                }
            })
            .collect();
        format_ident!("{}Module", pascal)
    };

    // Collect native function info from the module items
    let mut natives: Vec<NativeInfo> = Vec::new();

    if let Some((_, ref items)) = module.content {
        for item in items {
            if let Item::Fn(func) = item {
                if let Some(mut native) = extract_native_attr(&func.attrs)? {
                    native.fn_ident = func.sig.ident.clone();
                    natives.push(native);
                }
            }
        }
    }

    // Build the NativeDef entries
    let native_defs: Vec<_> = natives
        .iter()
        .map(|n| -> syn::Result<TokenStream> {
            let name_str = &n.native_name;
            let fn_ident = &n.fn_ident;
            let arity = n.arity;
            let effects = effect_tokens(&n.effects)?;
            let capabilities = capability_tokens(&n.capabilities)?;
            let behavior = behavior_tokens(&n.behavior)?;
            let cancellation = cancellation_tokens(&n.cancellation)?;
            let resource = resource_tokens(&n.resource)?;
            let async_start = match &n.async_adapter {
                Some(adapter) => {
                    let adapter = format_ident!("{adapter}");
                    quote! { Some(#mod_ident::#adapter) }
                }
                None => quote! { None },
            };
            Ok(quote! {
                ::akron::module::NativeDef {
                    name: #name_str,
                    func: #mod_ident::#fn_ident,
                    arity: #arity as isize,
                    effects: #effects,
                    capabilities: #capabilities,
                    behavior: #behavior,
                    cancellation: #cancellation,
                    resource: #resource,
                    async_start: #async_start,
                }
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let module_name_str = &attrs.name;

    // Emit the module (with #[ach_native] functions processed by their own macro)
    // plus the generated struct + trait impl
    Ok(quote! {
        #module

        #[derive(Debug, Clone, Copy)]
        #mod_vis struct #struct_name;

        impl ::akron::module::NativeModule for #struct_name {
            fn name(&self) -> &'static str {
                #module_name_str
            }

            fn natives(&self) -> Vec<::akron::module::NativeDef> {
                vec![
                    #(#native_defs),*
                ]
            }
        }
    })
}
