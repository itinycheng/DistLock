use crate::parse_args::DistLockArgs;
use impl_lock::generate;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::ItemFn;
use syn::Result;

mod impl_lock;
mod parse_args;

#[proc_macro_attribute]
pub fn dist_lock(head: TokenStream, body: TokenStream) -> TokenStream {
	let lock_args = parse_macro_input!(head as DistLockArgs);
	let function = parse_macro_input!(body as ItemFn);
	parse(lock_args, function).unwrap_or_else(to_compile_error).into()
}

fn parse(lock_args: DistLockArgs, function: ItemFn) -> Result<proc_macro2::TokenStream> {
	let dist_lock = generate(lock_args)?;
	let fn_vis = function.vis;
	let fn_body = function.block;
	let fn_sig = function.sig;
	let fn_name = fn_sig.ident;
	let fn_generics = fn_sig.generics;
	let fn_inputs = fn_sig.inputs;
	let fn_output = fn_sig.output;
	Ok(quote! {
		#fn_vis fn #fn_name #fn_generics (#fn_inputs) #fn_output{
			#dist_lock;
			#fn_body
		}
	})
}

fn to_compile_error(e: syn::Error) -> proc_macro2::TokenStream {
	e.to_compile_error()
}