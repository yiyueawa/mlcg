use std::collections::{HashMap, HashSet};

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::manifest::{InstructionSpec, Manifest};

const RESERVED_OWNER_PREFIX: &str = "\0reserved:";
const RESERVED_ITEM_SYMBOLS: &[&str] = &["Arg", "OutputArg", "LabelArg"];

pub(crate) fn generate(manifest: &Manifest) -> TokenStream {
    if let Err(error) = validate_manifest(manifest) {
        return quote! {
            compile_error!(#error);
        };
    }

    let _version = &manifest.version;
    let _source_tags: Vec<_> = manifest
        .instructions
        .iter()
        .map(|spec| (&spec.family, &spec.variant))
        .collect();
    let output_structs = manifest
        .instructions
        .iter()
        .filter(|spec| spec.outputs.len() > 1)
        .map(generate_output_struct);
    let structs = manifest
        .instructions
        .iter()
        .map(generate_instruction_struct);
    let processor_exts = manifest.instructions.iter().map(generate_processor_ext);
    let value_exts = manifest
        .instructions
        .iter()
        .filter(|spec| !spec.receiver.is_empty())
        .map(generate_value_ext);
    let prelude_exports = manifest.instructions.iter().flat_map(|spec| {
        let processor_trait = processor_trait_name(spec);
        let value_trait = value_trait_name(spec);
        let output_struct = output_struct_name(spec);
        let output_exports = (spec.outputs.len() > 1).then_some(quote! { #output_struct });
        if spec.receiver.is_empty() {
            vec![quote! { #processor_trait }]
        } else {
            vec![quote! { #processor_trait }, quote! { #value_trait }]
        }
        .into_iter()
        .chain(output_exports)
    });

    quote! {
        #[derive(Clone)]
        pub enum Arg<P> {
            Value(::mlcg_core::Value<P>),
            Raw(::std::string::String),
        }

        pub struct OutputArg<P>(::mlcg_core::Value<P>);

        impl<P> ::std::clone::Clone for OutputArg<P> {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }

        #[derive(Clone)]
        pub struct LabelArg<P>(::mlcg_core::Label<P>);

        impl<P> ::std::fmt::Debug for Arg<P> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Arg::Value(value) => f.debug_tuple("Value").field(value).finish(),
                    Arg::Raw(raw) => f.debug_tuple("Raw").field(raw).finish(),
                }
            }
        }

        impl<P, T> ::std::convert::From<::mlcg_core::Value<P, T>> for Arg<P> {
            fn from(value: ::mlcg_core::Value<P, T>) -> Self { Self::Value(value.erase_type()) }
        }

        impl<P, T> ::std::convert::From<&::mlcg_core::Value<P, T>> for Arg<P> {
            fn from(value: &::mlcg_core::Value<P, T>) -> Self { Self::Value(value.erase_type()) }
        }

        impl<P> ::std::fmt::Debug for OutputArg<P> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_tuple("Output").field(&self.0).finish()
            }
        }

        impl<P, T> ::std::convert::From<::mlcg_core::Value<P, T>> for OutputArg<P> {
            fn from(value: ::mlcg_core::Value<P, T>) -> Self { Self(value.erase_type()) }
        }

        impl<P, T> ::std::convert::From<&::mlcg_core::Value<P, T>> for OutputArg<P> {
            fn from(value: &::mlcg_core::Value<P, T>) -> Self { Self(value.erase_type()) }
        }

        macro_rules! impl_arg_from_display {
            ($($type:ty),* $(,)?) => {
                $(
                    impl<P> ::std::convert::From<$type> for Arg<P> {
                        fn from(value: $type) -> Self { Self::Raw(value.to_string()) }
                    }

                    impl<P> ::std::convert::From<&$type> for Arg<P> {
                        fn from(value: &$type) -> Self { Self::Raw(value.to_string()) }
                    }
                )*
            };
        }

        impl_arg_from_display!(
            i8, i16, i32, i64, i128, isize,
            u8, u16, u32, u64, u128, usize,
            bool, f32, f64,
        );

        impl<P> ::std::convert::From<&str> for Arg<P> {
            fn from(value: &str) -> Self { Self::Raw(value.to_string()) }
        }

        impl<P> ::std::convert::From<&::std::string::String> for Arg<P> {
            fn from(value: &::std::string::String) -> Self { Self::Raw(value.clone()) }
        }

        impl<P> ::std::convert::From<::std::string::String> for Arg<P> {
            fn from(value: ::std::string::String) -> Self { Self::Raw(value) }
        }

        impl<P> ::std::fmt::Debug for LabelArg<P> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_tuple("Label").field(&self.0).finish()
            }
        }

        impl<P> ::std::convert::From<::mlcg_core::Label<P>> for LabelArg<P> {
            fn from(value: ::mlcg_core::Label<P>) -> Self { Self(value) }
        }

        impl<P> ::std::convert::From<&::mlcg_core::Label<P>> for LabelArg<P> {
            fn from(value: &::mlcg_core::Label<P>) -> Self { Self(value.clone()) }
        }

        fn push_arg<P>(tokens: &mut ::std::vec::Vec<::mlcg_core::PartialToken<P>>, arg: &Arg<P>) {
            match arg {
                Arg::Value(value) => tokens.push(::mlcg_core::PartialToken::value(value.clone())),
                Arg::Raw(raw) => tokens.push(::mlcg_core::PartialToken::raw(raw.clone())),
            }
        }

        fn push_output_arg<P>(tokens: &mut ::std::vec::Vec<::mlcg_core::PartialToken<P>>, arg: &OutputArg<P>) {
            tokens.push(::mlcg_core::PartialToken::value(arg.0.clone()));
        }

        fn push_label_arg<P>(tokens: &mut ::std::vec::Vec<::mlcg_core::PartialToken<P>>, arg: &LabelArg<P>) {
            tokens.push(::mlcg_core::PartialToken::label(arg.0.clone()));
        }

        #(#output_structs)*
        #(#structs)*
        #(#processor_exts)*
        #(#value_exts)*

        pub mod prelude {
            pub use super::{Arg, OutputArg, LabelArg, #(#prelude_exports,)*};
        }
    }
}

fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    validate_instruction_names(manifest)?;
    validate_parameter_names(manifest)?;
    validate_item_symbols(manifest)?;
    validate_processor_methods(manifest)?;
    validate_value_methods(manifest)?;

    for spec in &manifest.instructions {
        validate_instruction_symbols(spec)?;
    }

    Ok(())
}

fn validate_instruction_names(manifest: &Manifest) -> Result<(), String> {
    for spec in &manifest.instructions {
        if spec.rust_name.is_empty() {
            return Err("instruction has empty rust_name".to_string());
        }
    }

    Ok(())
}

fn validate_parameter_names(manifest: &Manifest) -> Result<(), String> {
    for spec in &manifest.instructions {
        validate_named_parameters(spec, "input", &spec.inputs)?;
        validate_named_parameters(spec, "output", &spec.outputs)?;
        validate_named_parameters(spec, "label", &spec.labels)?;
    }

    Ok(())
}

fn validate_named_parameters(
    spec: &InstructionSpec,
    kind: &str,
    names: &[String],
) -> Result<(), String> {
    if names.iter().any(String::is_empty) {
        return Err(format!(
            "instruction `{}` has empty {kind} parameter name",
            spec.rust_name
        ));
    }

    Ok(())
}

fn validate_item_symbols(manifest: &Manifest) -> Result<(), String> {
    let mut seen = HashMap::new();
    for symbol in RESERVED_ITEM_SYMBOLS {
        reserve_manifest_symbol(&mut seen, symbol);
    }

    for spec in &manifest.instructions {
        record_manifest_symbol(&mut seen, "item", struct_name(spec).to_string(), spec)?;
        record_manifest_symbol(
            &mut seen,
            "item",
            processor_trait_name(spec).to_string(),
            spec,
        )?;

        if !spec.receiver.is_empty() {
            record_manifest_symbol(&mut seen, "item", value_trait_name(spec).to_string(), spec)?;
        }

        if spec.outputs.len() > 1 {
            record_manifest_symbol(
                &mut seen,
                "item",
                output_struct_name(spec).to_string(),
                spec,
            )?;
        }
    }

    Ok(())
}

fn reserve_manifest_symbol(seen: &mut HashMap<String, String>, name: &str) {
    seen.insert(name.to_string(), format!("{RESERVED_OWNER_PREFIX}{name}"));
}

fn validate_processor_methods(manifest: &Manifest) -> Result<(), String> {
    let mut seen = HashMap::new();

    for spec in &manifest.instructions {
        record_manifest_symbol(
            &mut seen,
            "processor method",
            safe_ident(&spec.rust_name).to_string(),
            spec,
        )?;

        if !spec.outputs.is_empty() {
            record_manifest_symbol(
                &mut seen,
                "processor method",
                safe_ident(&format!("{}_into", spec.rust_name)).to_string(),
                spec,
            )?;
        }
    }

    Ok(())
}

fn validate_value_methods(manifest: &Manifest) -> Result<(), String> {
    let mut seen = HashMap::new();

    for spec in manifest
        .instructions
        .iter()
        .filter(|spec| !spec.receiver.is_empty())
    {
        record_manifest_symbol(
            &mut seen,
            "value method",
            safe_ident(&spec.rust_name).to_string(),
            spec,
        )?;

        if !spec.outputs.is_empty() {
            record_manifest_symbol(
                &mut seen,
                "value method",
                safe_ident(&format!("{}_into", spec.rust_name)).to_string(),
                spec,
            )?;
        }
    }

    Ok(())
}

fn validate_instruction_symbols(spec: &InstructionSpec) -> Result<(), String> {
    validate_unique_instruction_names(
        "instruction field",
        spec,
        placeholders(spec)
            .into_iter()
            .map(|placeholder| placeholder.to_string()),
    )?;

    validate_unique_instruction_names(
        "processor auto parameter",
        spec,
        auto_processor_params(spec)
            .into_iter()
            .map(|param| safe_ident(&param).to_string()),
    )?;

    if spec.outputs.len() > 1 {
        validate_unique_instruction_names(
            "output field",
            spec,
            spec.outputs
                .iter()
                .map(|output| safe_ident(output).to_string()),
        )?;
    }

    validate_unique_instruction_names(
        "processor explicit parameter",
        spec,
        explicit_processor_params(spec)
            .into_iter()
            .map(|param| safe_ident(&param).to_string()),
    )?;

    if !spec.receiver.is_empty() {
        validate_unique_instruction_names(
            "value parameter",
            spec,
            value_params(spec)
                .into_iter()
                .map(|param| safe_ident(&param).to_string()),
        )?;
        validate_unique_instruction_names(
            "value explicit parameter",
            spec,
            explicit_value_params(spec)
                .into_iter()
                .map(|param| safe_ident(&param).to_string()),
        )?;
    }

    validate_placeholder_roles(spec)?;

    Ok(())
}

fn validate_placeholder_roles(spec: &InstructionSpec) -> Result<(), String> {
    let placeholders = emit_placeholders(&spec.emit);
    let mut roles = Vec::new();
    if !spec.receiver.is_empty() {
        roles.push(spec.receiver.as_str());
    }
    roles.extend(spec.inputs.iter().map(String::as_str));
    roles.extend(spec.outputs.iter().map(String::as_str));
    roles.extend(spec.labels.iter().map(String::as_str));

    let mut seen_roles = HashSet::new();
    for role in &roles {
        if !seen_roles.insert(*role) {
            return Err(format!(
                "instruction `{}` classifies parameter `{role}` more than once",
                spec.rust_name
            ));
        }
    }

    for placeholder in placeholders {
        if !roles.iter().any(|role| role == &placeholder) {
            return Err(format!(
                "instruction `{}` emits placeholder `${placeholder}` that is not classified as receiver, input, output, or label",
                spec.rust_name
            ));
        }
    }

    for role in roles {
        if !spec.emit.iter().any(|token| token == &format!("${role}")) {
            return Err(format!(
                "instruction `{}` classifies non-emitted parameter `{role}`",
                spec.rust_name
            ));
        }
    }

    Ok(())
}

fn record_manifest_symbol(
    seen: &mut HashMap<String, String>,
    kind: &str,
    name: String,
    spec: &InstructionSpec,
) -> Result<(), String> {
    if let Some(previous) = seen.insert(name.clone(), spec.rust_name.clone()) {
        if let Some(reserved) = previous.strip_prefix(RESERVED_OWNER_PREFIX) {
            return Err(format!(
                "generated {kind} `{name}` for instruction `{}` collides with reserved generated helper `{reserved}`",
                spec.rust_name
            ));
        }
        return Err(format!(
            "generated {kind} `{name}` for instruction `{}` collides with instruction `{previous}`",
            spec.rust_name
        ));
    }

    Ok(())
}

fn validate_unique_instruction_names(
    kind: &str,
    spec: &InstructionSpec,
    names: impl IntoIterator<Item = String>,
) -> Result<(), String> {
    let mut seen = HashSet::new();

    for name in names {
        if !seen.insert(name.clone()) {
            return Err(format!(
                "generated {kind} `{name}` appears more than once in instruction `{}`",
                spec.rust_name
            ));
        }
    }

    Ok(())
}

fn generate_output_struct(spec: &InstructionSpec) -> TokenStream {
    let output_struct = output_struct_name(spec);
    let field_idents: Vec<_> = spec
        .outputs
        .iter()
        .map(|output| safe_ident(output))
        .collect();
    let field_generics: Vec<_> = spec
        .outputs
        .iter()
        .map(|output| param_generic_ident(output))
        .collect();
    let fields: Vec<_> = field_idents
        .iter()
        .map(|field| quote! { pub #field: ::mlcg_core::Value<P> })
        .collect();
    let tuple_types: Vec<_> = field_idents
        .iter()
        .map(|_| quote! { ::mlcg_core::Value<P> })
        .collect();
    let tuple_ref_types: Vec<_> = field_idents
        .iter()
        .map(|_| quote! { &::mlcg_core::Value<P> })
        .collect();
    let tuple_borrow_types: Vec<_> = field_idents
        .iter()
        .map(|_| quote! { &'a ::mlcg_core::Value<P> })
        .collect();
    let tuple_fields = field_idents.iter();
    let tuple_ref_fields = field_idents.iter();
    let constructor_params = field_idents
        .iter()
        .zip(field_generics.iter())
        .map(|(field, generic)| quote! { #field: #generic });
    let constructor_where: Vec<_> = field_generics
        .iter()
        .map(|generic| quote! { #generic: ::std::convert::Into<OutputArg<P>> })
        .collect();
    let constructor_fields = field_idents
        .iter()
        .map(|field| quote! { #field: #field.into().0 });
    let from_tuple_types = field_generics.iter();
    let borrowed_tuple_types = field_generics.iter();
    let borrowed_tuple_where: Vec<_> = field_generics
        .iter()
        .map(|generic| quote! { &'a #generic: ::std::convert::Into<OutputArg<P>> })
        .collect();
    let from_tuple_fields = (0..field_idents.len()).map(syn::Index::from);
    let from_borrowed_tuple_fields = (0..field_idents.len()).map(syn::Index::from);

    quote! {
        #[allow(non_snake_case)]
        pub struct #output_struct<P> {
            #(#fields,)*
        }

        impl<P> ::std::clone::Clone for #output_struct<P> {
            fn clone(&self) -> Self {
                Self {
                    #(#field_idents: self.#field_idents.clone(),)*
                }
            }
        }

        impl<P> ::std::fmt::Debug for #output_struct<P> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct(stringify!(#output_struct))
                    #(.field(stringify!(#field_idents), &self.#field_idents))*
                    .finish()
            }
        }

        impl<P> #output_struct<P> {
            pub fn new<#(#field_generics,)*>(#(#constructor_params,)*) -> Self
            where
                #(#constructor_where,)*
            {
                Self {
                    #(#constructor_fields,)*
                }
            }

            pub fn into_tuple(self) -> (#(#tuple_types,)*) {
                (#(self.#tuple_fields,)*)
            }

            pub fn as_tuple(&self) -> (#(#tuple_ref_types,)*) {
                (#(&self.#tuple_ref_fields,)*)
            }
        }

        impl<P, #(#field_generics,)*> ::std::convert::From<(#(#from_tuple_types,)*)> for #output_struct<P>
        where
            #(#constructor_where,)*
        {
            fn from(value: (#(#field_generics,)*)) -> Self {
                Self::new(#(value.#from_tuple_fields,)*)
            }
        }

        impl<'a, P, #(#field_generics,)*> ::std::convert::From<&'a (#(#borrowed_tuple_types,)*)> for #output_struct<P>
        where
            #(#borrowed_tuple_where,)*
        {
            fn from(value: &'a (#(#field_generics,)*)) -> Self {
                Self::new(#(&value.#from_borrowed_tuple_fields,)*)
            }
        }

        impl<P> ::std::convert::From<&#output_struct<P>> for #output_struct<P> {
            fn from(value: &#output_struct<P>) -> Self {
                value.clone()
            }
        }

        impl<P> ::std::convert::From<#output_struct<P>> for (#(#tuple_types,)*) {
            fn from(value: #output_struct<P>) -> Self {
                value.into_tuple()
            }
        }

        impl<'a, P> ::std::convert::From<&'a #output_struct<P>> for (#(#tuple_borrow_types,)*) {
            fn from(value: &'a #output_struct<P>) -> Self {
                value.as_tuple()
            }
        }
    }
}

fn generate_instruction_struct(spec: &InstructionSpec) -> TokenStream {
    let struct_name = struct_name(spec);
    let fields = placeholders(spec);
    let field_defs: Vec<_> = fields
        .iter()
        .map(|field| {
            if is_output_field(spec, field) {
                quote! { #field: OutputArg<P> }
            } else if is_label_field(spec, field) {
                quote! { #field: LabelArg<P> }
            } else {
                quote! { #field: Arg<P> }
            }
        })
        .collect();
    let lower_steps: Vec<_> = spec
        .emit
        .iter()
        .map(|token| {
            if let Some(name) = token.strip_prefix('$') {
                let field = safe_ident(name);
                if spec.labels.iter().any(|label| label == name) {
                    quote! { push_label_arg(&mut tokens, &self.#field); }
                } else if spec.outputs.iter().any(|output| output == name) {
                    quote! { push_output_arg(&mut tokens, &self.#field); }
                } else {
                    quote! { push_arg(&mut tokens, &self.#field); }
                }
            } else {
                quote! { tokens.push(::mlcg_core::PartialToken::raw(#token)); }
            }
        })
        .collect();

    quote! {
        #[derive(Clone)]
        pub struct #struct_name<P> {
            _processor: ::std::marker::PhantomData<fn() -> P>,
            #(#field_defs,)*
        }

        impl<P> ::std::fmt::Debug for #struct_name<P> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct(stringify!(#struct_name)).finish_non_exhaustive()
            }
        }

        impl<P> ::mlcg_core::Instruction<P> for #struct_name<P>
        where
            P: ::std::marker::Send + ::std::marker::Sync + 'static,
        {
            fn lower(
                &self,
                _ctx: &mut ::mlcg_core::LowerContext<P>,
                out: &mut ::mlcg_core::PartialProgram<P>,
            ) -> ::std::result::Result<(), ::mlcg_core::LowerError> {
                let mut tokens = ::std::vec::Vec::new();
                #(#lower_steps)*
                out.push_line(::mlcg_core::PartialLine::new(tokens));
                ::std::result::Result::Ok(())
            }
        }
    }
}

fn generate_processor_ext(spec: &InstructionSpec) -> TokenStream {
    let struct_name = struct_name(spec);
    let trait_name = processor_trait_name(spec);
    let method = safe_ident(&spec.rust_name);
    let into_method = safe_ident(&format!("{}_into", spec.rust_name));

    let explicit_params = explicit_processor_params(spec);
    let explicit_param_idents: Vec<_> = explicit_params
        .iter()
        .map(|name| safe_ident(name))
        .collect();
    let explicit_generics: Vec<_> = explicit_params
        .iter()
        .map(|name| param_generic_ident(name))
        .collect();
    let explicit_params_sig: Vec<_> = explicit_param_idents
        .iter()
        .zip(explicit_generics.iter())
        .map(|(ident, generic)| quote! { #ident: #generic })
        .collect();
    let explicit_where: Vec<_> = explicit_params
        .iter()
        .zip(explicit_generics.iter())
        .map(|(name, generic)| param_where(spec, name, generic))
        .collect();
    let explicit_fields: Vec<_> = placeholders(spec)
        .into_iter()
        .map(|field| quote! { #field: #field.into() })
        .collect();

    match spec.outputs.len() {
        0 => quote! {
            #[allow(clippy::too_many_arguments, non_snake_case)]
            pub trait #trait_name<P> {
                fn #method<#(#explicit_generics,)*>(&self, #(#explicit_params_sig,)*)
                where
                    #(#explicit_where,)*;
            }

            #[allow(clippy::too_many_arguments, non_snake_case)]
            impl<P> #trait_name<P> for ::mlcg_core::Processor<P>
            where
                P: ::std::marker::Send + ::std::marker::Sync + 'static,
            {
                fn #method<#(#explicit_generics,)*>(&self, #(#explicit_params_sig,)*)
                where
                    #(#explicit_where,)*
                {
                    self.push(#struct_name { _processor: ::std::marker::PhantomData, #(#explicit_fields,)* });
                }
            }
        },
        1 => {
            let auto_params = auto_processor_params(spec);
            let auto_param_idents: Vec<_> =
                auto_params.iter().map(|name| safe_ident(name)).collect();
            let auto_generics: Vec<_> = auto_params
                .iter()
                .map(|name| param_generic_ident(name))
                .collect();
            let auto_params_sig: Vec<_> = auto_param_idents
                .iter()
                .zip(auto_generics.iter())
                .map(|(ident, generic)| quote! { #ident: #generic })
                .collect();
            let auto_where: Vec<_> = auto_params
                .iter()
                .zip(auto_generics.iter())
                .map(|(name, generic)| param_where(spec, name, generic))
                .collect();
            let output_ident = safe_ident(&spec.outputs[0]);
            let call_args = auto_param_idents.iter();

            quote! {
                #[allow(clippy::too_many_arguments, non_snake_case)]
                pub trait #trait_name<P> {
                    fn #method<#(#auto_generics,)*>(&self, #(#auto_params_sig,)*) -> ::mlcg_core::Value<P>
                    where
                        #(#auto_where,)*;

                    fn #into_method<#(#explicit_generics,)*>(&self, #(#explicit_params_sig,)*)
                    where
                        #(#explicit_where,)*;
                }

                #[allow(clippy::too_many_arguments, non_snake_case)]
                impl<P> #trait_name<P> for ::mlcg_core::Processor<P>
                where
                    P: ::std::marker::Send + ::std::marker::Sync + 'static,
                {
                    fn #method<#(#auto_generics,)*>(&self, #(#auto_params_sig,)*) -> ::mlcg_core::Value<P>
                    where
                        #(#auto_where,)*
                    {
                        let #output_ident = self.new_value();
                        <Self as #trait_name<P>>::#into_method(self, #output_ident.clone(), #(#call_args,)*);
                        #output_ident
                    }

                    fn #into_method<#(#explicit_generics,)*>(&self, #(#explicit_params_sig,)*)
                    where
                        #(#explicit_where,)*
                    {
                        self.push(#struct_name { _processor: ::std::marker::PhantomData, #(#explicit_fields,)* });
                    }
                }
            }
        }
        _ => {
            let output_struct = output_struct_name(spec);
            let outputs_arg = format_ident!("OutputsArg");
            let outputs_param = format_ident!("__mlcg_outputs");
            let auto_params = auto_processor_params(spec);
            let auto_param_idents: Vec<_> =
                auto_params.iter().map(|name| safe_ident(name)).collect();
            let auto_generics: Vec<_> = auto_params
                .iter()
                .map(|name| param_generic_ident(name))
                .collect();
            let auto_params_sig: Vec<_> = auto_param_idents
                .iter()
                .zip(auto_generics.iter())
                .map(|(ident, generic)| quote! { #ident: #generic })
                .collect();
            let auto_where: Vec<_> = auto_params
                .iter()
                .zip(auto_generics.iter())
                .map(|(name, generic)| param_where(spec, name, generic))
                .collect();
            let output_idents: Vec<_> = spec.outputs.iter().map(|name| safe_ident(name)).collect();
            let explicit_params: Vec<_> = explicit_processor_params(spec)
                .into_iter()
                .filter(|param| !spec.outputs.iter().any(|output| output == param))
                .collect();
            let explicit_param_idents: Vec<_> = explicit_params
                .iter()
                .map(|name| safe_ident(name))
                .collect();
            let explicit_generics: Vec<_> = explicit_params
                .iter()
                .map(|name| param_generic_ident(name))
                .collect();
            let explicit_params_sig: Vec<_> = explicit_param_idents
                .iter()
                .zip(explicit_generics.iter())
                .map(|(ident, generic)| quote! { #ident: #generic })
                .collect();
            let explicit_where: Vec<_> = explicit_params
                .iter()
                .zip(explicit_generics.iter())
                .map(|(name, generic)| param_where(spec, name, generic))
                .collect();
            let output_allocations = output_idents
                .iter()
                .map(|output| quote! { let #output = self.new_value(); });
            let output_call_args = output_idents
                .iter()
                .map(|output| quote! { #output: #output.clone() });
            let output_fields = output_idents.iter().map(|output| quote! { #output });
            let call_args = auto_param_idents.iter();
            let explicit_fields: Vec<_> = placeholders(spec)
                .into_iter()
                .map(|field| {
                    if output_idents.iter().any(|output| output == &field) {
                        quote! { #field: #outputs_param.#field.clone().into() }
                    } else {
                        quote! { #field: #field.into() }
                    }
                })
                .collect();

            quote! {
                #[allow(clippy::too_many_arguments, non_snake_case)]
                pub trait #trait_name<P> {
                    fn #method<#(#auto_generics,)*>(&self, #(#auto_params_sig,)*) -> #output_struct<P>
                    where
                        #(#auto_where,)*;

                    fn #into_method<#outputs_arg, #(#explicit_generics,)*>(&self, #outputs_param: #outputs_arg, #(#explicit_params_sig,)*)
                    where
                        #outputs_arg: ::std::convert::Into<#output_struct<P>>,
                        #(#explicit_where,)*;
                }

                #[allow(clippy::too_many_arguments, non_snake_case)]
                impl<P> #trait_name<P> for ::mlcg_core::Processor<P>
                where
                    P: ::std::marker::Send + ::std::marker::Sync + 'static,
                {
                    fn #method<#(#auto_generics,)*>(&self, #(#auto_params_sig,)*) -> #output_struct<P>
                    where
                        #(#auto_where,)*
                    {
                        #(#output_allocations)*
                        <Self as #trait_name<P>>::#into_method(self, #output_struct { #(#output_call_args,)* }, #(#call_args,)*);
                        #output_struct { #(#output_fields,)* }
                    }

                    fn #into_method<#outputs_arg, #(#explicit_generics,)*>(&self, #outputs_param: #outputs_arg, #(#explicit_params_sig,)*)
                    where
                        #outputs_arg: ::std::convert::Into<#output_struct<P>>,
                        #(#explicit_where,)*
                    {
                        let #outputs_param = #outputs_param.into();
                        self.push(#struct_name { _processor: ::std::marker::PhantomData, #(#explicit_fields,)* });
                    }
                }
            }
        }
    }
}

fn generate_value_ext(spec: &InstructionSpec) -> TokenStream {
    let struct_name = struct_name(spec);
    let trait_name = value_trait_name(spec);
    let method = safe_ident(&spec.rust_name);
    let into_method = safe_ident(&format!("{}_into", spec.rust_name));
    let receiver = safe_ident(&spec.receiver);
    let value_params = value_params(spec);
    let input_idents: Vec<_> = value_params.iter().map(|name| safe_ident(name)).collect();
    let input_generics: Vec<_> = value_params
        .iter()
        .map(|name| param_generic_ident(name))
        .collect();
    let input_params_sig: Vec<_> = input_idents
        .iter()
        .zip(input_generics.iter())
        .map(|(ident, generic)| quote! { #ident: #generic })
        .collect();
    let input_where: Vec<_> = value_params
        .iter()
        .zip(input_generics.iter())
        .map(|(name, generic)| param_where(spec, name, generic))
        .collect();
    let output_idents: Vec<_> = spec.outputs.iter().map(|name| safe_ident(name)).collect();
    let fields: Vec<_> = placeholders(spec)
        .into_iter()
        .filter(|field| !output_idents.iter().any(|output| output == field))
        .map(|field| {
            if field == receiver {
                quote! { #field: self.clone().into() }
            } else {
                quote! { #field: #field.into() }
            }
        })
        .collect();

    let explicit_all_value_params = explicit_value_params(spec);
    let explicit_value_param_idents: Vec<_> = explicit_all_value_params
        .iter()
        .map(|name| safe_ident(name))
        .collect();
    let explicit_value_generics: Vec<_> = explicit_all_value_params
        .iter()
        .map(|name| param_generic_ident(name))
        .collect();
    let explicit_value_params_sig: Vec<_> = explicit_value_param_idents
        .iter()
        .zip(explicit_value_generics.iter())
        .map(|(ident, generic)| quote! { #ident: #generic })
        .collect();
    let explicit_value_where: Vec<_> = explicit_all_value_params
        .iter()
        .zip(explicit_value_generics.iter())
        .map(|(name, generic)| param_where(spec, name, generic))
        .collect();
    let explicit_value_fields: Vec<_> = placeholders(spec)
        .into_iter()
        .map(|field| {
            if field == receiver {
                quote! { #field: self.clone().into() }
            } else {
                quote! { #field: #field.into() }
            }
        })
        .collect();

    if spec.outputs.len() == 1 {
        let output_ident = safe_ident(&spec.outputs[0]);
        quote! {
            #[allow(clippy::too_many_arguments, non_snake_case)]
            pub trait #trait_name<P> {
                fn #method<#(#input_generics,)*>(&self, #(#input_params_sig,)*) -> ::mlcg_core::Value<P>
                where
                    #(#input_where,)*;

                fn #into_method<#(#explicit_value_generics,)*>(&self, #(#explicit_value_params_sig,)*)
                where
                    #(#explicit_value_where,)*;
            }

            #[allow(clippy::too_many_arguments, non_snake_case)]
            impl<P, T> #trait_name<P> for ::mlcg_core::Value<P, T>
            where
                P: ::std::marker::Send + ::std::marker::Sync + 'static,
            {
                fn #method<#(#input_generics,)*>(&self, #(#input_params_sig,)*) -> ::mlcg_core::Value<P>
                where
                    #(#input_where,)*
                {
                    let #output_ident = self.handle().new_value();
                    self.handle().push(#struct_name {
                        _processor: ::std::marker::PhantomData,
                        #output_ident: #output_ident.clone().into(),
                        #(#fields,)*
                    });
                    #output_ident
                }

                fn #into_method<#(#explicit_value_generics,)*>(&self, #(#explicit_value_params_sig,)*)
                where
                    #(#explicit_value_where,)*
                {
                    self.handle().push(#struct_name { _processor: ::std::marker::PhantomData, #(#explicit_value_fields,)* });
                }
            }
        }
    } else if spec.outputs.is_empty() {
        quote! {
            #[allow(clippy::too_many_arguments)]
            pub trait #trait_name<P> {
                fn #method<#(#input_generics,)*>(&self, #(#input_params_sig,)*)
                where
                    #(#input_where,)*;
            }

            #[allow(clippy::too_many_arguments)]
            impl<P, T> #trait_name<P> for ::mlcg_core::Value<P, T>
            where
                P: ::std::marker::Send + ::std::marker::Sync + 'static,
            {
                fn #method<#(#input_generics,)*>(&self, #(#input_params_sig,)*)
                where
                    #(#input_where,)*
                {
                    self.handle().push(#struct_name { _processor: ::std::marker::PhantomData, #(#fields,)* });
                }
            }
        }
    } else {
        let output_struct = output_struct_name(spec);
        let outputs_arg = format_ident!("OutputsArg");
        let outputs_param = format_ident!("__mlcg_outputs");
        let output_allocations = output_idents
            .iter()
            .map(|output| quote! { let #output = handle.new_value(); });
        let output_instruction_fields = output_idents
            .iter()
            .map(|output| quote! { #output: #output.clone().into() });
        let output_fields = output_idents.iter().map(|output| quote! { #output });
        let explicit_multi_value_params: Vec<_> = explicit_value_params(spec)
            .into_iter()
            .filter(|param| !spec.outputs.iter().any(|output| output == param))
            .collect();
        let explicit_value_param_idents: Vec<_> = explicit_multi_value_params
            .iter()
            .map(|name| safe_ident(name))
            .collect();
        let explicit_value_generics: Vec<_> = explicit_multi_value_params
            .iter()
            .map(|name| param_generic_ident(name))
            .collect();
        let explicit_value_params_sig: Vec<_> = explicit_value_param_idents
            .iter()
            .zip(explicit_value_generics.iter())
            .map(|(ident, generic)| quote! { #ident: #generic })
            .collect();
        let explicit_value_where: Vec<_> = explicit_multi_value_params
            .iter()
            .zip(explicit_value_generics.iter())
            .map(|(name, generic)| param_where(spec, name, generic))
            .collect();
        let explicit_value_fields: Vec<_> = placeholders(spec)
            .into_iter()
            .map(|field| {
                if output_idents.iter().any(|output| output == &field) {
                    quote! { #field: #outputs_param.#field.clone().into() }
                } else if field == receiver {
                    quote! { #field: self.clone().into() }
                } else {
                    quote! { #field: #field.into() }
                }
            })
            .collect();

        quote! {
            #[allow(clippy::too_many_arguments, non_snake_case)]
            pub trait #trait_name<P> {
                fn #method<#(#input_generics,)*>(&self, #(#input_params_sig,)*) -> #output_struct<P>
                where
                    #(#input_where,)*;

                fn #into_method<#outputs_arg, #(#explicit_value_generics,)*>(&self, #outputs_param: #outputs_arg, #(#explicit_value_params_sig,)*)
                where
                    #outputs_arg: ::std::convert::Into<#output_struct<P>>,
                    #(#explicit_value_where,)*;
            }

            #[allow(clippy::too_many_arguments, non_snake_case)]
            impl<P, T> #trait_name<P> for ::mlcg_core::Value<P, T>
            where
                P: ::std::marker::Send + ::std::marker::Sync + 'static,
            {
                fn #method<#(#input_generics,)*>(&self, #(#input_params_sig,)*) -> #output_struct<P>
                where
                    #(#input_where,)*
                {
                    let handle = self.handle();
                    #(#output_allocations)*
                    handle.push(#struct_name {
                        _processor: ::std::marker::PhantomData,
                        #(#output_instruction_fields,)*
                        #(#fields,)*
                    });
                    #output_struct { #(#output_fields,)* }
                }

                fn #into_method<#outputs_arg, #(#explicit_value_generics,)*>(&self, #outputs_param: #outputs_arg, #(#explicit_value_params_sig,)*)
                where
                    #outputs_arg: ::std::convert::Into<#output_struct<P>>,
                    #(#explicit_value_where,)*
                {
                    let #outputs_param = #outputs_param.into();
                    self.handle().push(#struct_name { _processor: ::std::marker::PhantomData, #(#explicit_value_fields,)* });
                }
            }
        }
    }
}

fn param_where(spec: &InstructionSpec, name: &str, generic: &Ident) -> TokenStream {
    if spec.outputs.iter().any(|output| output == name) {
        quote! { #generic: ::std::convert::Into<OutputArg<P>> }
    } else if spec.labels.iter().any(|label| label == name) {
        quote! { #generic: ::std::convert::Into<LabelArg<P>> }
    } else {
        quote! { #generic: ::std::convert::Into<Arg<P>> }
    }
}

fn explicit_value_params(spec: &InstructionSpec) -> Vec<String> {
    let mut params = Vec::new();
    if spec.outputs.len() <= 1 {
        params.extend(spec.outputs.iter().cloned());
    }
    for label in &spec.labels {
        if !params.contains(label) {
            params.push(label.clone());
        }
    }
    for input in &spec.inputs {
        if !params.contains(input) {
            params.push(input.clone());
        }
    }
    params
}

fn value_params(spec: &InstructionSpec) -> Vec<String> {
    let mut params = Vec::new();
    for label in &spec.labels {
        if !params.contains(label) {
            params.push(label.clone());
        }
    }
    for input in &spec.inputs {
        if !params.contains(input) {
            params.push(input.clone());
        }
    }
    params
}

fn is_label_field(spec: &InstructionSpec, field: &Ident) -> bool {
    spec.labels.iter().any(|label| safe_ident(label) == *field)
}

fn is_output_field(spec: &InstructionSpec, field: &Ident) -> bool {
    spec.outputs
        .iter()
        .any(|output| safe_ident(output) == *field)
}

fn auto_processor_params(spec: &InstructionSpec) -> Vec<String> {
    let mut params = Vec::new();
    if !spec.receiver.is_empty() {
        params.push(spec.receiver.clone());
    }
    for label in &spec.labels {
        if !params.contains(label) {
            params.push(label.clone());
        }
    }
    for input in &spec.inputs {
        if !params.contains(input) {
            params.push(input.clone());
        }
    }
    params
}

fn explicit_processor_params(spec: &InstructionSpec) -> Vec<String> {
    let mut params = Vec::new();
    if spec.outputs.len() <= 1 {
        params.extend(spec.outputs.iter().cloned());
    }
    if !spec.receiver.is_empty() && !params.contains(&spec.receiver) {
        params.push(spec.receiver.clone());
    }
    for label in &spec.labels {
        if !params.contains(label) {
            params.push(label.clone());
        }
    }
    for input in &spec.inputs {
        if !params.contains(input) {
            params.push(input.clone());
        }
    }
    params
}

fn placeholders(spec: &InstructionSpec) -> Vec<Ident> {
    let mut names = Vec::<String>::new();
    for token in &spec.emit {
        if let Some(name) = token.strip_prefix('$') {
            if !names.iter().any(|existing| existing == name) {
                names.push(name.to_string());
            }
        }
    }
    names.iter().map(|name| safe_ident(name)).collect()
}

fn emit_placeholders(emit: &[String]) -> Vec<&str> {
    let mut placeholders = Vec::new();
    for token in emit {
        if let Some(placeholder) = token.strip_prefix('$') {
            if !placeholders.contains(&placeholder) {
                placeholders.push(placeholder);
            }
        }
    }
    placeholders
}

fn struct_name(spec: &InstructionSpec) -> Ident {
    to_pascal_ident(&spec.rust_name)
}

fn output_struct_name(spec: &InstructionSpec) -> Ident {
    format_ident!("{}Output", struct_name(spec))
}

fn processor_trait_name(spec: &InstructionSpec) -> Ident {
    format_ident!("Processor{}Ext", struct_name(spec))
}

fn value_trait_name(spec: &InstructionSpec) -> Ident {
    format_ident!("Value{}Ext", struct_name(spec))
}

fn param_generic_ident(name: &str) -> Ident {
    format_ident!("Mlcg{}Arg", to_pascal_string(name))
}

fn to_pascal_ident(name: &str) -> Ident {
    format_ident!("{}", to_pascal_string(name))
}

fn to_pascal_string(name: &str) -> String {
    let mut out = String::new();
    for part in name.split('_') {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    if is_rust_keyword(&out) {
        format!("Arg{out}")
    } else {
        out
    }
}

fn safe_ident(name: &str) -> Ident {
    if is_rust_keyword(name) {
        format_ident!("arg_{}", name)
    } else {
        format_ident!("{}", name)
    }
}

fn is_rust_keyword(name: &str) -> bool {
    matches!(
        name,
        "Self"
            | "as"
            | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "try"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "union"
    )
}
