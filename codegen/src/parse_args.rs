use proc_macro2::Span;
use syn::parenthesized;
use syn::parse::Parse;
use syn::ExprCall;
use syn::LitStr;
use syn::Token;

pub(crate) struct DistLockArgs {
	pub(crate) name: LitStr,
	pub(crate) at_most: LitStr,
	pub(crate) at_least: Option<LitStr>,
	pub(crate) connection: ExprCall,
}

impl Parse for DistLockArgs {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let mut name = None;
		let mut at_least = None;
		let mut at_most = None;
		let mut connection = None;
		while !input.is_empty() {
			let lookahead = input.lookahead1();
			if lookahead.peek(kw::name) {
				_ = input.parse::<kw::name>()?;
				_ = input.parse::<Token![=]>()?;
				name = Some(input.parse()?);
			} else if lookahead.peek(kw::at_most) {
				_ = input.parse::<kw::at_most>()?;
				_ = input.parse::<Token![=]>()?;
				at_most = Some(input.parse::<LitStr>()?);
			} else if lookahead.peek(kw::at_least) {
				_ = input.parse::<kw::at_least>()?;
				_ = input.parse::<Token![=]>()?;
				at_least = Some(input.parse::<LitStr>()?);
			} else if lookahead.peek(kw::connection) {
				_ = input.parse::<kw::connection>()?;
				let content;
				parenthesized!(content in input);
				connection = Some(content.parse::<ExprCall>()?);
			} else if lookahead.peek(Token![,]) {
				_ = input.parse::<Token![,]>()?;
			} else {
				return Err(lookahead.error());
			}
		}

		Ok(DistLockArgs {
			name: name.ok_or(syn::Error::new(Span::call_site(), "lock name not found"))?,
			at_most: at_most.ok_or(syn::Error::new(Span::call_site(), "at_most not found"))?,
			at_least,
			connection: connection
				.ok_or(syn::Error::new(Span::call_site(), "connection not found"))?,
		})
	}
}

mod kw {
	use syn::custom_keyword;

	custom_keyword!(name);
	custom_keyword!(at_least);
	custom_keyword!(at_most);
	custom_keyword!(connection);
}
