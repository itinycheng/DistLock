use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;
use syn::Result;

use crate::parse_args::DistLockArgs;

pub(crate) fn generate(lock_args: DistLockArgs) -> Result<TokenStream> {
	let name = lock_args.name.value();
	let at_most_string = lock_args.at_most.value();
	let at_most = at_most_string
		.parse::<humantime::Duration>()
		.map(|d| d.as_millis() as i64)
		.map_err(|_| {
			syn::Error::new(Span::call_site(), format!("can't prase at_most: {}", at_most_string))
		})?;

	let at_least_string = lock_args.at_least.map(|s| s.value()).unwrap_or("0s".to_string());
	let at_least = at_least_string
		.parse::<humantime::Duration>()
		.map(|d| d.as_millis() as i64)
		.map_err(|_| {
			syn::Error::new(Span::call_site(), format!("can't prase at_least: {}", at_most_string))
		})?;

	Ok(gen_lock_code(name, at_most, at_least, lock_args.transport))
}

#[cfg(feature = "redis")]
fn gen_lock_code(
	name: String,
	at_most_mills: i64,
	at_least_mills: i64,
	transport: Expr,
) -> TokenStream {
	let name_ident = Ident::new(&name, Span::call_site());
	quote! {
	   let mut #name_ident = {
			use ::dist_lock::core::DistLock;
			use ::dist_lock::core::LockConfig;
			use ::dist_lock::provider::RedisDriver;

			let lock_name = #name.to_string();
			let driver = RedisDriver::new(&lock_name, #transport);
			let config = LockConfig::from_mills(lock_name, #at_least_mills, #at_most_mills);
			DistLock::new(config, driver)
		}
	}
}

#[cfg(feature = "diesel")]
fn gen_lock_code(
	name: String,
	at_most_mills: i64,
	at_least_mills: i64,
	transport: Expr,
) -> TokenStream {
	let name_ident = Ident::new(&name, Span::call_site());
	quote! {
	   let mut #name_ident = {
			use ::dist_lock::core::DistLock;
			use ::dist_lock::core::LockConfig;
			use ::dist_lock::provider::DieselDriver;

			let lock_name = #name.to_string();
			let driver = DieselDriver::new(&lock_name, Some("t"), #transport);
			let config = LockConfig::from_mills(lock_name, #at_least_mills, #at_most_mills);
			DistLock::new(config, driver)
		}
	}
}
