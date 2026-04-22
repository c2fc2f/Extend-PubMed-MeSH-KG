//! # dispatch-derive
//!
//! A procedural macro crate that derives an async `dispatch` method on
//! [`clap`](https://docs.rs/clap) subcommand enums, eliminating the
//! boilerplate of hand-written `match` arms that delegate to per-module
//! `run` functions.
//!
//! ## How it works
//!
//! Annotate your subcommand enum with `#[derive(Dispatch)]`. For every
//! tuple variant whose sole field is a path type `some::module::SubArgs`,
//! the macro generates a match arm that calls `some::module::run(args).await`
//! and returns a [`std::process::ExitCode`].
//!
//! The last path segment (conventionally `SubArgs`) is stripped and `::run`
//! is appended automatically, so the dispatch target is always the `run`
//! function that lives alongside the args struct.
//!
//! ## Requirements
//!
//! | Requirement | Detail |
//! |---|---|
//! | Enum only | `#[derive(Dispatch)]` panics at compile-time on structs or unions |
//! | Tuple variants | Each variant must be a single-field tuple: `Foo(foo::SubArgs)` |
//! | Path types | The field type must be a plain path — no references, generics, etc. |
//! | `run` function | Each module must expose `pub async fn run(args: SubArgs) -> ExitCode` |
//!
//! ## Example
//!
//! ```rust,ignore
//! use dispatch_derive::Dispatch;
//!
//! mod stats {
//!     use clap::Args;
//!     use std::process::ExitCode;
//!
//!     #[derive(Args)]
//!     pub struct SubArgs { /* ... */ }
//!
//!     pub async fn run(args: SubArgs) -> ExitCode {
//!         ExitCode::SUCCESS
//!     }
//! }
//!
//! #[derive(clap::Subcommand, Dispatch)]
//! enum Command {
//!     Stats(stats::SubArgs),
//! }
//!
//! // The macro expands to:
//! //
//! // impl Command {
//! //     pub async fn dispatch(self) -> std::process::ExitCode {
//! //         match self {
//! //             Command::Stats(args) => stats::run(args).await,
//! //         }
//! //     }
//! // }
//! ```
//!
//! ## Errors
//!
//! All violations are reported as compile-time errors pointing at the
//! offending item:
//!
//! - Applying `#[derive(Dispatch)]` to a non-enum
//! - A variant that is not a single-field tuple
//! - A variant field whose type is not a plain path

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input,
    punctuated::{IntoIter, Punctuated},
    token::PathSep,
    Data, DeriveInput, Fields, Ident, Path, PathSegment, Type, TypePath,
};

/// Derives a `dispatch(self) -> std::process::ExitCode` async method on a
/// clap `Subcommand` enum.
///
/// Convention: each variant must be a single-field tuple whose type is
/// `some::module::SubArgs`. The last path segment (`SubArgs`) is stripped
/// and `::run` is appended, so `Stats(stats::SubArgs)` dispatches to
/// `stats::run(args).await`.
///
/// # Errors
///
/// Emits a compile-time error if:
/// - The target is not an enum.
/// - A variant is not a single-field tuple.
/// - A variant field type is not a plain path.
#[proc_macro_derive(Dispatch)]
pub fn derive_dispatch(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let enum_name: &Ident = &input.ident;

    let Data::Enum(data_enum) = input.data else {
        return syn::Error::new_spanned(&input.ident, "Dispatch only works on enums")
            .to_compile_error()
            .into();
    };

    let arms: Vec<TokenStream2> = data_enum
        .variants
        .into_iter()
        .map(|variant| {
            let variant_name: Ident = variant.ident;

            let Fields::Unnamed(fields) = variant.fields else {
                return syn::Error::new_spanned(
                    variant_name,
                    "Dispatch requires single-field tuple variants, e.g. Stats(stats::SubArgs)",
                )
                .to_compile_error();
            };

            if fields.unnamed.len() != 1 {
                return syn::Error::new_spanned(
                    variant_name,
                    "Dispatch requires exactly one field per variant",
                )
                .to_compile_error();
            }

            let field_ty: &Type = &fields.unnamed[0].ty;
            let Type::Path(TypePath { path, .. }) = &field_ty else {
                return syn::Error::new_spanned(
                    field_ty,
                    "Dispatch requires a plain path type (e.g. stats::SubArgs)",
                )
                .to_compile_error();
            };

            let module_path: Path = module_from_type_path(path);

            quote! {
                #enum_name::#variant_name(args) => #module_path::run(args).await,
            }
        })
        .collect();

    quote! {
        impl #enum_name {
            /// Executes the subcommand, returning its exit code.
            pub async fn dispatch(self) -> ::std::process::ExitCode {
                match self {
                    #(#arms)*
                }
            }
        }
    }
    .into()
}

/// Strips the last path segment from `some::module::SubArgs`
/// and returns `some::module` as a [`Path`].
///
/// # Panics
///
/// Panics (via [`syn::parse_quote!`]) if the path is a single bare segment
/// with no parent, producing an empty token stream. Callers should validate
/// that the path has at least two segments before calling this function.
fn module_from_type_path(path: &Path) -> Path {
    let mut segments: Punctuated<PathSegment, PathSep> = path.segments.clone();
    segments.pop();
    let segments: IntoIter<PathSegment> = segments.into_iter();
    syn::parse_quote! { #(#segments)::* }
}

