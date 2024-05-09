use proc_macro::TokenStream;

mod ebml;

#[proc_macro]
pub fn ebml_define(item: TokenStream) -> TokenStream {
	ebml::ebml_define(item.into()).into()
}
