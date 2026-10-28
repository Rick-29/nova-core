//! Input parsing and the two entry points shared by the `math_*` and
//! `real_*` macros.

use zyn::syn::{
    self, Expr, Ident, Token, Type,
    parse::{Parse, ParseStream},
};
use zyn::{ToTokens, zyn};

use crate::node::{Level, Render};

/// Everything needed to render a node: the number type and the error mapper.
pub struct Ctx {
    pub mapper: Expr,
    pub ty: Type,
}

/// `*_impl!(mapper, Type, expr)`
struct ImplInput {
    ctx: Ctx,
    expr: Expr,
}

impl Parse for ImplInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mapper = input.parse()?;
        input.parse::<Token![,]>()?;
        let ty = input.parse()?;
        input.parse::<Token![,]>()?;
        let expr = input.parse()?;
        Ok(ImplInput {
            ctx: Ctx { mapper, ty },
            expr,
        })
    }
}

/// `*_support!(name, mapper, Type)`
struct SupportInput {
    name: Ident,
    mapper: Expr,
    ty: Type,
}

impl Parse for SupportInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![,]>()?;
        let mapper = input.parse()?;
        input.parse::<Token![,]>()?;
        let ty = input.parse()?;
        Ok(SupportInput { name, mapper, ty })
    }
}

/// Body of `math_impl!` / `real_impl!`.
pub fn expand(input: proc_macro::TokenStream, level: Level) -> proc_macro::TokenStream {
    let ImplInput { ctx, expr } = syn::parse_macro_input!(input as ImplInput);
    match level.lower(expr) {
        Ok(node) => Render(&ctx, &node).to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Body of `math_support!` / `real_support!`: defines a local `macro_rules!`
/// that forwards to `::nova_ta_core::<target>!(mapper, Type, ..)`.
pub fn support(input: proc_macro::TokenStream, target: &str) -> proc_macro::TokenStream {
    let SupportInput { name, mapper, ty } = syn::parse_macro_input!(input as SupportInput);
    let target = zyn::format_ident!("{}", target);
    zyn!(
        macro_rules! {{ name }} {
            ($($t:tt)*) => {
                ::nova_ta_core::{{ target }}!({{ mapper }}, {{ ty }}, $($t)*)
            };
        }
    )
    .to_token_stream()
    .into()
}
