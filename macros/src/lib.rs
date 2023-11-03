use proc_macro::TokenStream;

mod ebml;

#[proc_macro]
pub fn ebml_element(item: TokenStream) -> TokenStream {
	ebml::ebml_element(item.into()).into()
}
