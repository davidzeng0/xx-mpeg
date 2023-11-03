use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
	parse::{self, *},
	punctuated::Punctuated,
	spanned::Spanned,
	*
};

fn is_primitive(ty: &str) -> bool {
	match ty {
		"vint" | "vfloat" | "u128" | "u64" | "u32" | "u16" | "u8" | "i128" | "i64" | "i32" |
		"i16" | "i8" | "f64" | "f32" => true,
		_ => false
	}
}

fn is_variable_sized(ty: &str) -> bool {
	match ty {
		"vint" | "vfloat" | "String" | "Vec" => true,
		_ => false
	}
}

fn data_type(ty: &Ident, args: &PathArguments) -> Option<(Type, String)> {
	if !args.is_empty() {
		return None;
	}

	let type_str = &ty.to_string() as &str;
	let (data_type, reader_type) = match type_str {
		"vint" => Some((quote! { u64 }, "u64")),
		"vfloat" => Some((quote! { f64 }, "f64")),
		"String" => Some((quote! { String}, "string")),
		prim if is_primitive(prim) => Some((quote! { #ty }, prim)),
		_ => None
	}?;

	Some((
		parse_quote_spanned! { ty.span() => #data_type },
		reader_type.to_string()
	))
}

fn var_occur_type(ty: &Ident, args: &PathArguments) -> Option<(Type, u64, Option<u64>)> {
	loop {
		let (min, max) = match &ty.to_string() as &str {
			"Option" => (0, Some(1)),
			"Vec" => (0, None),
			_ => break
		};

		let PathArguments::AngleBracketed(angle) = &args else {
			break;
		};
		let GenericArgument::Type(ty) = angle.args.first()? else {
			break;
		};

		return Some((ty.clone(), min, max));
	}

	None
}

struct Data {
	name: Ident,
	ty: Ident,
	default: Option<Literal>,

	data_type: Type,
	reader_type: String,
	is_primitive: bool,
	is_variable_sized: bool
}

struct Child {
	name: Ident,
	ty: Type,

	min_occur: u64,
	max_occur: Option<u64>
}

enum Parse {
	Data(Vec<Data>),
	Master(Vec<Child>)
}

struct Element {
	attrs: Vec<Attribute>,
	struct_token: token::Struct,
	name: Ident,
	brace_token: token::Brace,
	id: LitInt,
	parse: Parse,
	post_parse: Option<ImplItemFn>
}

fn make_parse(
	span: Span, fields: Punctuated<(Field, Option<Literal>), Token![,]>
) -> Result<Parse> {
	let mut data_fields = Vec::new();
	let mut children = Vec::new();

	for (field, default) in &fields {
		loop {
			let Type::Path(path) = &field.ty else {
				return Err(Error::new(
					field.span(),
					"Only primitive and struct types allowed"
				));
			};

			if path.path.leading_colon.is_some() || path.path.segments.len() != 1 {
				break;
			}

			let path = path.path.segments.first().unwrap();
			let ty = &path.ident;

			if let Some((data_type, reader_type)) = data_type(ty, &path.arguments) {
				let type_str = ty.to_string();

				data_fields.push(Data {
					name: field.ident.clone().unwrap(),
					ty: ty.clone(),
					default: default.clone(),
					data_type,
					reader_type,
					is_primitive: is_primitive(&type_str),
					is_variable_sized: is_variable_sized(&type_str)
				});

				break;
			}

			if default.is_some() {
				return Err(Error::new(
					field.span(),
					"Only primitive types may have default values"
				));
			}

			if let Some((ty, min, max)) = var_occur_type(ty, &path.arguments) {
				let mut is_binary = false;

				loop {
					if min != 0 || max != None {
						break;
					}

					let Type::Path(path) = &ty else {
						break;
					};

					if path.path.leading_colon.is_some() || path.path.segments.len() != 1 {
						break;
					}

					let path = path.path.segments.first().unwrap();
					let ty = &path.ident;

					let Some(_) = data_type(ty, &path.arguments) else {
						break;
					};

					if ty != "u8" {
						break;
					}

					is_binary = true;
					data_fields.push(Data {
						name: field.ident.clone().unwrap(),
						ty: ty.clone(),
						default: default.clone(),
						data_type: field.ty.clone(),
						reader_type: "bytes".to_string(),
						is_primitive: false,
						is_variable_sized: true
					});

					break;
				}

				if is_binary {
					break;
				}

				children.push(Child {
					name: field.ident.clone().unwrap(),
					ty,

					min_occur: min,
					max_occur: max
				});

				break;
			}

			children.push(Child {
				name: field.ident.clone().unwrap(),
				ty: field.ty.clone(),

				min_occur: 1,
				max_occur: Some(1)
			});

			break;
		}
	}

	if !data_fields.is_empty() && !children.is_empty() {
		return Err(Error::new(
			span,
			"a master element cannot have parsable fields"
		));
	}

	Ok(if children.is_empty() {
		Parse::Data(data_fields)
	} else {
		Parse::Master(children)
	})
}

impl parse::Parse for Element {
	fn parse(input: ParseStream) -> Result<Self> {
		let attrs = input.call(Attribute::parse_outer)?;

		let struct_token = input.parse::<Token![struct]>()?;

		let name = input.parse::<Ident>()?;
		let content;
		let brace_token = braced!(content in input);

		content.parse::<Token![const]>()?;

		if content.parse::<Ident>()? != "ID" {
			return Err(content.error("unexpected const"));
		}

		content.parse::<Token![=]>()?;

		let id = content.parse::<LitInt>()?;

		content.parse::<Token![;]>()?;

		let post_parse = if input.peek(Token![async]) || input.peek(Token![fn]) {
			let func: ImplItemFn = input.parse()?;

			if func.sig.ident != "post_parse" {
				return Err(input.error("unexpected function"));
			}

			Some(func)
		} else {
			None
		};

		let fields = content.parse_terminated(
			|input| {
				let field = Field::parse_named(input)?;

				let default = if input.peek(Token![=]) {
					input.parse::<Token![=]>()?;

					Some(input.parse()?)
				} else {
					None
				};

				Ok((field, default))
			},
			Token![,]
		)?;

		let parse = make_parse(content.span(), fields)?;

		Ok(Self {
			attrs,
			struct_token,
			name,
			brace_token,
			id,
			parse,
			post_parse
		})
	}
}

impl Element {
	fn expand(&self) -> TokenStream {
		let mut fields: Punctuated<Field, Token![,]> = Punctuated::new();

		let mut parse_lines = Vec::new();
		let mut defaults = Vec::new();
		let mut required_post_parse = Vec::new();

		let mut expanded = Vec::new();

		let mut add_field = |ident: Ident, ty: Type| {
			fields.push(Field {
				attrs: Vec::new(),
				colon_token: Default::default(),
				vis: Visibility::Public(Default::default()),
				ident: Some(ident),
				mutability: FieldMutability::None,
				ty
			});
		};

		let (name, id) = (&self.name, &self.id);

		match &self.parse {
			Parse::Data(fields) => {
				for field in fields {
					let ty = &field.data_type;
					let ty = quote_spanned! { field.ty.span() => #ty };

					add_field(field.name.clone(), parse_quote! { #ty });

					let read_func = if field.is_primitive {
						format_ident!("read_{}_be", field.ty)
					} else {
						format_ident!("read_{}", field.reader_type, span = field.ty.span())
					};

					let arg = if field.is_variable_sized {
						quote! { element.size as usize }
					} else {
						quote! {}
					};

					let name = &field.name;

					parse_lines.push(quote! {
						this.#name = parser.reader().#read_func(#arg).await?;
					});

					if let Some(default) = &field.default {
						let ty = &field.data_type;

						if field.is_primitive {
							defaults.push(quote! {
								#name: #default
							});
						} else {
							defaults.push(quote! {
								#name: #ty::from(#default)
							});
						}
					} else {
						defaults.push(quote! {
							#name: std::default::Default::default()
						});
					}
				}
			}

			Parse::Master(fields) => {
				let mut ids = Vec::new();
				let mut element_handlers = Vec::new();

				for field in fields {
					let ty = &field.ty;
					let name = &field.name;

					let parsed = quote! { <#ty as ParseExt>::parse(parser, element).await? };

					let (wrap, parsed) = match (field.min_occur, field.max_occur) {
						(0, Some(1)) => (quote! { Option<#ty> }, quote! { = Some(#parsed) }),

						(0, None) => (quote! { Vec<#ty> }, quote! { .push(#parsed) }),

						(1, Some(1)) => {
							required_post_parse.push(quote! {
								self.#name.post_parse()?;
							});

							(quote! { #ty }, quote! { = #parsed })
						}

						_ => unreachable!()
					};

					add_field(field.name.clone(), parse_quote! { #wrap });

					element_handlers.push(quote! {
						<#ty as Parse>::ID => {
							parser.pre_parse::<#ty>(element, PhantomData)?;

							self.#name #parsed;
						}
					});

					ids.push(quote! { <#ty as Parse>::ID });

					defaults.push(quote! {
						#name: std::default::Default::default()
					});
				}

				parse_lines.push(quote! {
					let master = MasterElement {
						element: element.clone(),
						children: Self::CHILDREN
					};

					parser.read_children(&master, |parser, element| {
						this.handle_child(parser, element).await
					}).await?;
				});

				expanded.push(quote! {
					#[async_trait_impl]
					impl MasterParse for #name {
						const CHILDREN: &'static [ElementId] = &[#(#ids),*];

						async fn handle_child<P: Parser>(&mut self, parser: &mut P, element: &Element) -> Result<()> {
							match element.id {
								#(#element_handlers)*
								_ => unreachable!()
							}

							Ok(())
						}
					}
				});
			}
		}

		let post_parse = if let Some(post_parse) = &self.post_parse {
			post_parse.clone()
		} else {
			parse_quote! {
				fn post_parse(&mut self) -> Result<()> {
					#(#required_post_parse)*

					Ok(())
				}
			}
		};

		expanded.push(quote! {
			#[async_trait_impl]
			impl Parse for #name {
				const ID: ElementId = make_id(#id);
				const NAME: &'static str = stringify!(#name);

				async fn parse<P: Parser>(parser: &mut P, element: &Element) -> Result<Self> {
					let mut this = Self::default();

					#(#parse_lines);*

					this.post_parse()?;

					Ok(this)
				}

				#post_parse
			}
		});

		let item_struct = ItemStruct {
			attrs: self.attrs.clone(),
			vis: Visibility::Public(Default::default()),
			struct_token: self.struct_token.clone(),
			ident: self.name.clone(),
			generics: Default::default(),
			fields: syn::Fields::Named(FieldsNamed {
				brace_token: self.brace_token.clone(),
				named: fields.clone()
			}),
			semi_token: None
		};

		expanded.push(quote! {
			#[derive(Debug)]
			#item_struct
		});

		expanded.push(quote! {
			impl std::default::Default for #name {
				fn default() -> Self {
					Self {
						#(#defaults),*
					}
				}
			}
		});

		quote! {
			#(#expanded)*
		}
	}
}

pub fn ebml_element(item: TokenStream) -> TokenStream {
	let element = match parse2::<Element>(item) {
		Ok(element) => element.expand(),
		Err(err) => return err.to_compile_error()
	};

	element
}
