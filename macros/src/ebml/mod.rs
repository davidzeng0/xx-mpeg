use convert_case::{Case, Casing};
use pluralizer::pluralize;

use super::*;
#[derive(Clone)]
struct EbmlField {
	field: Field,
	rename: Option<String>,
	id: Option<LitInt>,
	default: Option<Expr>,
	flatten: bool
}

struct Element {
	attrs: Vec<Attribute>,
	vis: Visibility,
	name: Ident,
	fields_named: bool,
	fields: Vec<EbmlField>,
	check: Option<ImplItemFn>
}

enum Define {
	Element(Element),
	Enum(ItemEnum)
}

fn parse_field_rest(input: &ParseBuffer<'_>, mut field: Field) -> Result<EbmlField> {
	let mut id = None;
	let mut default = None;
	let mut rename = None;
	let mut flatten = false;

	if input.parse::<Option<Token![@]>>()?.is_some() {
		id = Some(input.parse()?);
	}

	if input.parse::<Option<Token![=]>>()?.is_some() {
		default = Some(input.parse()?);
	}

	if field.attrs.remove_path("flatten").is_some() {
		flatten = true;
	}

	if let Some(attr) = field.attrs.remove_name_value("rename") {
		let Expr::Lit(ExprLit { lit: Lit::Str(str), .. }) = &attr.value else {
			return Err(Error::new_spanned(attr.value, "Expected a str"));
		};

		rename = Some(str.value());
	}

	Ok(EbmlField { field, rename, id, default, flatten })
}

fn parse_named_fields(input: ParseBuffer<'_>) -> Result<Punctuated<EbmlField, Token![,]>> {
	input.parse_terminated(
		|input| {
			let field = Field::parse_named(input)?;

			parse_field_rest(input, field)
		},
		Token![,]
	)
}

fn parse_unnamed_fields(input: ParseBuffer<'_>) -> Result<Punctuated<EbmlField, Token![,]>> {
	input.parse_terminated(
		|input| {
			let field = Field::parse_unnamed(input)?;

			parse_field_rest(input, field)
		},
		Token![,]
	)
}

impl Parse for Define {
	fn parse(input: ParseStream<'_>) -> Result<Self> {
		let attrs = input.call(Attribute::parse_outer)?;
		let vis = input.parse::<Visibility>()?;

		Ok(if input.peek(Token![enum]) {
			let mut parsed = ItemEnum::parse(input)?;

			parsed.attrs = attrs;
			parsed.vis = vis;

			Self::Enum(parsed)
		} else {
			input.parse::<Token![struct]>()?;

			let name = input.parse::<Ident>()?;
			let content;

			let (named, fields) = if input.peek(token::Paren) {
				parenthesized!(content in input);

				let fields = parse_unnamed_fields(content)?;

				input.parse::<Token![;]>()?;

				(false, fields)
			} else {
				braced!(content in input);

				(true, parse_named_fields(content)?)
			};

			let check = if input.peek(Token![async]) || input.peek(Token![fn]) {
				let func: ImplItemFn = input.parse()?;

				if func.sig.ident != "check" {
					return Err(Error::new_spanned(func, "Unexpected function"));
				}

				Some(func)
			} else {
				None
			};

			Self::Element(Element {
				attrs,
				vis,
				name,
				fields_named: named,
				fields: fields.into_iter().collect(),
				check
			})
		})
	}
}

fn base_path(span: Option<Span>) -> TokenStream {
	quote_spanned! { span.unwrap_or_else(Span::call_site) =>
		::xx_mpeg::demuxer::mkv::ebml
	}
}

fn generate_struct(
	element: &Element, name: &Ident, modify: impl Fn(&mut EbmlField)
) -> TokenStream {
	let fields: Punctuated<_, Token![,]> = element
		.fields
		.iter()
		.map(|field| {
			let mut field = field.clone();

			modify(&mut field);

			field.field
		})
		.collect();

	let (attrs, vis) = (&element.attrs, &element.vis);

	let fields = if element.fields_named {
		quote! { { #fields } }
	} else {
		quote! { ( #fields ); }
	};

	quote! {
		#(#attrs)*
		#[allow(clippy::option_option)]
		#vis struct #name #fields
	}
}

fn partial_ident(name: &Ident) -> Ident {
	format_ident!("Partial{}", name)
}

#[allow(clippy::missing_panics_doc)]
fn partial_path(path: &Path) -> Path {
	let mut path = path.clone();
	let last = path.segments.last_mut().unwrap();

	last.ident = partial_ident(&last.ident);
	path
}

#[allow(clippy::missing_panics_doc)]
fn generate_partial(element: &Element) -> Result<TokenStream> {
	let mut finalize = Vec::new();

	for EbmlField {
		field: Field { ident, ty, .. }, default, flatten, ..
	} in &element.fields
	{
		let Type::Path(TypePath { qself: None, .. }) = ty else {
			return Err(Error::new_spanned(ty, "Must be a path"));
		};

		if *flatten {
			finalize.push(quote! { #ident: self.#ident.finalize()? });

			continue;
		}

		let mut ident = ident.clone().unwrap();

		ident.set_span(ty.span());

		let default = if let Some(default) = default {
			let default = quote_spanned! { default.span() =>
				|| #default
			};

			quote_spanned! { default.span() =>
				get_or_default(self.#ident, #default)
			}
		} else {
			quote_spanned! { ty.span() =>
				get(self.#ident)?
			}
		};

		let base = base_path(Some(ty.span()));

		finalize.push(quote! { #ident: #base::internal::FieldInit::#default });
	}

	let ident = &element.name;
	let partial = partial_ident(ident);

	let item = generate_struct(element, &partial, |field| {
		let ty = &field.field.ty;

		if field.flatten {
			let Type::Path(TypePath { path, .. }) = ty else {
				unreachable!();
			};

			let partial = partial_path(path);

			field.field.ty = parse_quote! { #partial };
		} else {
			field.field.ty = parse_quote! { ::std::option::Option<#ty> };
		}
	});

	Ok(quote! {
		#[derive(Default)]
		#item

		impl #partial {
			pub fn finalize(self) -> ::xx_core::error::Result<#ident> {
				Ok(#ident {
					#(#finalize),*
				})
			}
		}
	})
}

fn generate_master(element: &Element) -> Result<(TokenStream, TokenStream)> {
	let base = base_path(None);
	let name = &element.name;
	let partial = partial_ident(name);

	let mut ids = Vec::new();
	let mut id_consts = Vec::new();
	let mut element_handlers = Vec::new();
	let mut flattened = Vec::new();
	let mut flattened_ids = Vec::new();

	for EbmlField {
		field: Field { ident, ty, .. },
		rename,
		id,
		flatten,
		..
	} in &element.fields
	{
		let Some(mut ident) = ident.clone() else {
			let msg = "Master elements require named fields";

			return Err(Error::new_spanned(ty, msg));
		};

		ident.set_span(ty.span());

		let base = base_path(Some(ty.span()));

		if *flatten {
			flattened.push(quote_spanned! { ty.span() =>
				#base::parse::MasterElementExt::handle_child(&mut self.#ident, reader, header).await?
			});

			let Type::Path(TypePath { qself: None, path }) = ty else {
				return Err(Error::new_spanned(ty, "Must be a path"));
			};

			let partial = partial_path(path);

			flattened_ids.push(quote! {
				<#partial as #base::parse::MasterElement>::CHILDREN
			});

			continue;
		}

		let field_name = rename.clone().unwrap_or_else(|| ident.to_string());

		let child_name = field_name.to_case(Case::Pascal);
		let child_name_singular = pluralize(field_name.as_ref(), 1, false).to_case(Case::Pascal);

		let id = quote! { #id };
		let id_const = format_ident!(
			"{}_ID",
			field_name.to_case(Case::ScreamingSnake),
			span = id.span()
		);

		let child_name = if child_name == child_name_singular {
			quote! { #child_name }
		} else {
			quote_spanned! { ty.span() =>
				if <::std::option::Option<#ty> as #base::internal::FieldMeta>::MULTIPLE {
					#child_name_singular
				} else {
					#child_name
				}
			}
		};

		let parse = quote_spanned! { ty.span() =>
			#base::internal::FieldInit::insert(
				&mut self.#ident,
				<
					<::std::option::Option<#ty> as #base::internal::FieldMeta>
						::Element as #base::parse::ElementExt
				>::parse(reader, header).await?
			)?;
		};

		let mut name = name.clone();

		name.set_span(id.span());

		let id_path = quote_spanned! { id.span() =>
			#name::#id_const
		};

		element_handlers.push(quote! {
			#id_path => {
				#base::parse::EbmlReaderExt::trace_element(
					reader,
					#child_name,
					header
				);

				#parse
			}
		});

		ids.push(quote! { #base::parse::make_id(#id) });
		id_consts.push(id_const);
	}

	Ok((
		quote! {
			<#partial as #base::parse::ElementExt>::parse(reader, header)
				.await?.finalize()?
		},
		quote! {
			impl #name {
				#(pub const #id_consts: #base::EbmlId = #ids;)*
			}

			#[::xx_pulse::asynchronous]
			impl #base::parse::Element for #partial {
				async fn parse<R>(reader: &mut R, header: &#base::parse::ElemHdr) -> ::xx_core::error::Result<Self>
				where
					R: #base::parse::EbmlReader
				{
					use ::std::default::Default;
					use #base::parse::{MasterElemHdr, MasterElement, MasterElementExt, ElemHdr, EbmlReaderExt};

					let master = MasterElemHdr {
						element: *header,
						children: <Self as MasterElement>::CHILDREN
					};

					let mut this = Default::default();

					EbmlReaderExt::read_children(
						reader,
						&master,
						|reader: &mut R, header: &ElemHdr| async move {
							MasterElementExt::handle_child(&mut this, reader, header).await
						}
					).await?;

					Ok(this)
				}
			}

			#[::xx_pulse::asynchronous]
			impl #base::parse::MasterElement for #partial {
				const CHILDREN: &'static [#base::EbmlId] =
					::xx_mpeg::constcat::concat_slices!(
						[#base::EbmlId]:
						&[#(#name::#id_consts),*],
						#(#flattened_ids)*
					);

				async fn handle_child<R>(
					&mut self,
					reader: &mut R,
					header: &#base::parse::ElemHdr
				) -> ::xx_core::error::Result<bool>
				where
					R: #base::parse::EbmlReader
				{
					match header.id {
						#(#element_handlers)*
						_ => {
							let mut matched = false;

							#(if !matched {
								matched = #flattened;
							})*;

							return Ok(matched)
						}
					}

					Ok(true)
				}
			}
		}
	))
}

fn generate_simple(element: &Element) -> TokenStream {
	let base = base_path(None);
	let mut fields = Vec::new();
	let mut parses = Vec::new();

	for (index, EbmlField { field: Field { ident, ty, .. }, .. }) in
		element.fields.iter().enumerate()
	{
		let name = if let Some(ident) = ident {
			let name = format_ident!("{}", ident, span = Span::mixed_site());

			fields.push(quote! { #ident: #name });
			name
		} else {
			let name = format_ident!("_{}", index);
			let member = Member::Unnamed(index.into());

			fields.push(quote! { #member: #name });
			name
		};

		parses.push(quote_spanned! { ty.span() =>
			let #name = {
				if header.size != #base::UNKNOWN_SIZE {
					header.size = header.size.checked_sub(reader.position() - header.offset)
						.ok_or(::xx_mpeg::FormatError::ReadOverflow)?;
				}

				header.offset = reader.position();

				<#ty as #base::parse::ElementExt>::parse(reader, &header).await?
			};
		});
	}

	quote! {
		let mut header = *header;

		#(#parses)*;

		Self { #(#fields),* }
	}
}

fn generate_default(element: &Element) -> TokenStream {
	let mut fields = Vec::new();

	for EbmlField { field: Field { ident, .. }, default, .. } in &element.fields {
		let default = default
			.clone()
			.unwrap_or_else(|| parse_quote! { ::std::default::Default::default() });

		fields.push(quote! { #ident: #default });
	}

	let name = &element.name;

	quote! {
		impl ::std::default::Default for #name {
			fn default() -> Self {
				Self {
					#(#fields),*
				}
			}
		}
	}
}

impl Element {
	fn expand(&self) -> Result<TokenStream> {
		let base = base_path(None);
		let all_id = self.fields.iter().all(|f| f.id.is_some() || f.flatten);
		let none_id = self.fields.iter().all(|f| f.id.is_none() && !f.flatten);

		if !all_id && !none_id {
			return Err(Error::new(
				Span::call_site(),
				"Cannot mix master element and simple element fields"
			));
		}

		let mut items = Vec::new();
		let (name, check) = (&self.name, &self.check);

		items.push(generate_struct(self, &self.name, |field| {
			let ty = &field.field.ty;

			field.field.ty = parse_quote_spanned! { ty.span() =>
				<::std::option::Option<#ty> as #base::internal::FieldMeta>::Output
			};
		}));

		let parse = if all_id {
			let (parses, elem_handlers) = generate_master(self)?;

			items.push(generate_partial(self)?);
			items.push(elem_handlers);
			parses
		} else {
			generate_simple(self)
		};

		if none_id && self.fields.iter().any(|field| field.default.is_some()) {
			items.push(generate_default(self));
		}

		items.push(quote! {
			#[::xx_pulse::asynchronous]
			impl #base::parse::Element for #name {
				async fn parse<R>(reader: &mut R, header: &#base::parse::ElemHdr) -> Result<Self>
				where
					R: #base::parse::EbmlReader
				{
					mod __xx_internal_post_parse_verify_support {
						pub trait Check {
							fn check(&mut self) -> ::xx_core::error::Result<()> {
								Ok(())
							}
						}
					}

					impl __xx_internal_post_parse_verify_support::Check for #name {
						#check
					}

					let mut parsed = { #parse };

					__xx_internal_post_parse_verify_support::Check::check(&mut parsed)?;

					Ok(parsed)
				}
			}
		});

		Ok(quote! { #(#items)* })
	}
}

fn transform_enum(mut item: ItemEnum) -> Result<TokenStream> {
	let base = base_path(None);
	let ident = &item.ident;

	item.attrs.push(parse_quote! {
		#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, ::num_derive::FromPrimitive)]
	});

	let parse = if item.attrs.remove_path("bitflags").is_some() {
		item.attrs.push(parse_quote! { #[bitflags] });

		None
	} else {
		let repr = item
			.attrs
			.remove_list("repr")
			.ok_or_else(|| Error::new(Span::call_site(), "Expected a repr"))?;

		let MacroDelimiter::Paren(_) = repr.delimiter else {
			return Err(Error::new_spanned(repr, "Expected parenthesis"));
		};

		let repr = repr.tokens;

		Some(quote_spanned! { repr.span() =>
			#[::xx_pulse::asynchronous]
			impl #base::parse::Element for #ident {
				async fn parse<R>(reader: &mut R, header: &#base::parse::ElemHdr) -> Result<Self>
				where
					R: #base::parse::EbmlReader
				{
					let repr = <#repr as #base::parse::ElementExt>::parse(reader, header).await?;

					{
						use ::std::convert::Into;
						use #base::{internal::EnumRepr, parse::EbmlError};

						EnumRepr::convert(repr).ok_or_else(
							|| Into::into(EbmlError::InvalidVariant)
						)
					}
				}
			}
		})
	};

	Ok(quote! {
		#item
		#parse
	})
}

pub fn ebml_define(item: TokenStream) -> Result<TokenStream> {
	let def = parse2::<Define>(item)?;

	match def {
		Define::Element(element) => element.expand(),
		Define::Enum(item) => transform_enum(item)
	}
}
