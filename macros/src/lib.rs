use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::parse::*;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use xx_macro_support::*;

mod ebml;

declare_proc_macro! {
	pub fn ebml_define(item: TokenStream) -> Result<TokenStream> {
		ebml::ebml_define(item)
	}
}
