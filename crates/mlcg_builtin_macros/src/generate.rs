use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::manifest::{InstructionSpec, Manifest};

pub(crate) fn generate(manifest: &Manifest) -> TokenStream {
    let _version = &manifest.version;
    let _source_tags: Vec<_> = manifest
        .instructions
        .iter()
        .map(|spec| (&spec.family, &spec.variant))
        .collect();
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
        if spec.receiver.is_empty() {
            vec![quote! { #processor_trait }]
        } else {
            vec![quote! { #processor_trait }, quote! { #value_trait }]
        }
    });

    quote! {
        use mlcg_core::{Instruction, LowerContext, PartialLine, PartialProgram, PartialToken, Processor, Value};

        #[derive(Clone)]
        pub enum Arg<P> {
            Value(Value<P>),
            Raw(String),
        }

        impl<P> ::std::fmt::Debug for Arg<P> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Arg::Value(value) => f.debug_tuple("Value").field(value).finish(),
                    Arg::Raw(raw) => f.debug_tuple("Raw").field(raw).finish(),
                }
            }
        }

        impl<P> From<Value<P>> for Arg<P> {
            fn from(value: Value<P>) -> Self { Self::Value(value) }
        }

        impl<P> From<&Value<P>> for Arg<P> {
            fn from(value: &Value<P>) -> Self { Self::Value(value.clone()) }
        }

        impl<P> From<i32> for Arg<P> {
            fn from(value: i32) -> Self { Self::Raw(value.to_string()) }
        }

        impl<P> From<i64> for Arg<P> {
            fn from(value: i64) -> Self { Self::Raw(value.to_string()) }
        }

        impl<P> From<usize> for Arg<P> {
            fn from(value: usize) -> Self { Self::Raw(value.to_string()) }
        }

        impl<P> From<f64> for Arg<P> {
            fn from(value: f64) -> Self { Self::Raw(value.to_string()) }
        }

        impl<P> From<&str> for Arg<P> {
            fn from(value: &str) -> Self { Self::Raw(value.to_string()) }
        }

        impl<P> From<String> for Arg<P> {
            fn from(value: String) -> Self { Self::Raw(value) }
        }

        fn push_arg<P>(tokens: &mut Vec<PartialToken<P>>, arg: &Arg<P>) {
            match arg {
                Arg::Value(value) => tokens.push(PartialToken::value(value.clone())),
                Arg::Raw(raw) => tokens.push(PartialToken::raw(raw.clone())),
            }
        }

        #(#structs)*
        #(#processor_exts)*
        #(#value_exts)*

        pub mod prelude {
            pub use super::{Arg, #(#prelude_exports,)*};
        }
    }
}

fn generate_instruction_struct(spec: &InstructionSpec) -> TokenStream {
    let struct_name = struct_name(spec);
    let fields = placeholders(spec);
    let field_defs: Vec<_> = fields
        .iter()
        .map(|field| quote! { #field: Arg<P> })
        .collect();
    let lower_steps: Vec<_> = spec
        .emit
        .iter()
        .map(|token| {
            if let Some(name) = token.strip_prefix('$') {
                let field = safe_ident(name);
                quote! { push_arg(&mut tokens, &self.#field); }
            } else {
                quote! { tokens.push(PartialToken::raw(#token)); }
            }
        })
        .collect();

    quote! {
        #[derive(Clone)]
        pub struct #struct_name<P> {
            #(#field_defs,)*
        }

        impl<P> ::std::fmt::Debug for #struct_name<P> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct(stringify!(#struct_name)).finish_non_exhaustive()
            }
        }

        impl<P> Instruction<P> for #struct_name<P>
        where
            P: Send + Sync + 'static,
        {
            fn lower(
                &self,
                _ctx: &mut LowerContext<P>,
                out: &mut PartialProgram<P>,
            ) -> Result<(), mlcg_core::LowerError> {
                let mut tokens = Vec::new();
                #(#lower_steps)*
                out.push_line(PartialLine::new(tokens));
                Ok(())
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
        .map(|name| format_ident!("{}Arg", to_pascal_string(name)))
        .collect();
    let explicit_params_sig: Vec<_> = explicit_param_idents
        .iter()
        .zip(explicit_generics.iter())
        .map(|(ident, generic)| quote! { #ident: #generic })
        .collect();
    let explicit_where: Vec<_> = explicit_generics
        .iter()
        .map(|generic| quote! { #generic: Into<Arg<P>> })
        .collect();
    let explicit_fields: Vec<_> = placeholders(spec)
        .into_iter()
        .map(|field| quote! { #field: #field.into() })
        .collect();

    if spec.outputs.len() == 1 {
        let auto_params = spec.inputs.clone();
        let auto_param_idents: Vec<_> = auto_params.iter().map(|name| safe_ident(name)).collect();
        let auto_generics: Vec<_> = auto_params
            .iter()
            .map(|name| format_ident!("{}Arg", to_pascal_string(name)))
            .collect();
        let auto_params_sig: Vec<_> = auto_param_idents
            .iter()
            .zip(auto_generics.iter())
            .map(|(ident, generic)| quote! { #ident: #generic })
            .collect();
        let auto_where: Vec<_> = auto_generics
            .iter()
            .map(|generic| quote! { #generic: Into<Arg<P>> })
            .collect();
        let output_ident = safe_ident(&spec.outputs[0]);
        let call_args = auto_param_idents.iter();

        quote! {
            pub trait #trait_name<P> {
                fn #method<#(#auto_generics,)*>(&self, #(#auto_params_sig,)*) -> Value<P>
                where
                    #(#auto_where,)*;

                fn #into_method<#(#explicit_generics,)*>(&self, #(#explicit_params_sig,)*)
                where
                    #(#explicit_where,)*;
            }

            impl<P> #trait_name<P> for Processor<P>
            where
                P: Send + Sync + 'static,
            {
                fn #method<#(#auto_generics,)*>(&self, #(#auto_params_sig,)*) -> Value<P>
                where
                    #(#auto_where,)*
                {
                    let #output_ident = self.new_value();
                    self.#into_method(#output_ident.clone(), #(#call_args,)*);
                    #output_ident
                }

                fn #into_method<#(#explicit_generics,)*>(&self, #(#explicit_params_sig,)*)
                where
                    #(#explicit_where,)*
                {
                    self.push(#struct_name { #(#explicit_fields,)* });
                }
            }
        }
    } else if spec.outputs.is_empty() {
        quote! {
            pub trait #trait_name<P> {
                fn #method<#(#explicit_generics,)*>(&self, #(#explicit_params_sig,)*)
                where
                    #(#explicit_where,)*;
            }

            impl<P> #trait_name<P> for Processor<P>
            where
                P: Send + Sync + 'static,
            {
                fn #method<#(#explicit_generics,)*>(&self, #(#explicit_params_sig,)*)
                where
                    #(#explicit_where,)*
                {
                    self.push(#struct_name { #(#explicit_fields,)* });
                }
            }
        }
    } else {
        panic!(
            "instruction {} has {} auto outputs; only zero or one is supported",
            spec.rust_name,
            spec.outputs.len()
        );
    }
}

fn generate_value_ext(spec: &InstructionSpec) -> TokenStream {
    let struct_name = struct_name(spec);
    let trait_name = value_trait_name(spec);
    let method = safe_ident(&spec.rust_name);
    let receiver = safe_ident(&spec.receiver);
    let input_idents: Vec<_> = spec.inputs.iter().map(|name| safe_ident(name)).collect();
    let input_generics: Vec<_> = spec
        .inputs
        .iter()
        .map(|name| format_ident!("{}Arg", to_pascal_string(name)))
        .collect();
    let input_params_sig: Vec<_> = input_idents
        .iter()
        .zip(input_generics.iter())
        .map(|(ident, generic)| quote! { #ident: #generic })
        .collect();
    let input_where: Vec<_> = input_generics
        .iter()
        .map(|generic| quote! { #generic: Into<Arg<P>> })
        .collect();
    let fields: Vec<_> = placeholders(spec)
        .into_iter()
        .map(|field| {
            if field == receiver {
                quote! { #field: self.clone().into() }
            } else {
                quote! { #field: #field.into() }
            }
        })
        .collect();

    quote! {
        pub trait #trait_name<P> {
            fn #method<#(#input_generics,)*>(&self, #(#input_params_sig,)*)
            where
                #(#input_where,)*;
        }

        impl<P> #trait_name<P> for Value<P>
        where
            P: Send + Sync + 'static,
        {
            fn #method<#(#input_generics,)*>(&self, #(#input_params_sig,)*)
            where
                #(#input_where,)*
            {
                self.handle().push(#struct_name { #(#fields,)* });
            }
        }
    }
}

fn explicit_processor_params(spec: &InstructionSpec) -> Vec<String> {
    let mut params = Vec::new();
    params.extend(spec.outputs.iter().cloned());
    if !spec.receiver.is_empty() && !params.contains(&spec.receiver) {
        params.push(spec.receiver.clone());
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

fn struct_name(spec: &InstructionSpec) -> Ident {
    to_pascal_ident(&spec.rust_name)
}

fn processor_trait_name(spec: &InstructionSpec) -> Ident {
    format_ident!("{}ProcessorExt", struct_name(spec))
}

fn value_trait_name(spec: &InstructionSpec) -> Ident {
    format_ident!("{}ValueExt", struct_name(spec))
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
    out
}

fn safe_ident(name: &str) -> Ident {
    match name {
        "type" | "match" | "ref" | "self" | "crate" | "super" | "in" | "where" => {
            format_ident!("arg_{}", name)
        }
        _ => format_ident!("{}", name),
    }
}
