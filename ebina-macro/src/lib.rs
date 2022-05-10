extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::token::Token;

use std::collections::HashSet as Set;

use syn::{parse_macro_input, parse_quote, Expr, Ident, ItemFn, Local, Pat, Stmt, Token, FnArg, PatType, NestedMeta, Lit};
use syn::parse::{Parse, ParseStream, Result};

// Proc macro attribute for adding tracking to a function
#[proc_macro_attribute]
pub fn tracking(args: TokenStream, input: TokenStream) -> TokenStream {
	let args = parse_macro_input!(args as syn::AttributeArgs);

	let name = if let NestedMeta::Lit(Lit::Str(v)) = args.first().unwrap() {
		v
	} else {
		panic!("tracking attribute must be a string literal");
	};
	let ItemFn { attrs, vis, sig, block } = parse_macro_input!(input as ItemFn);
	let syn::Signature {constness, asyncness, unsafety, abi, fn_token, ident, generics, paren_token: _, inputs, variadic, output } = sig;

	let first_input_ident = if let FnArg::Typed(PatType { pat, .. }) = inputs.first().unwrap() {
		if let Pat::Ident(i) = *pat.clone() {
			i.ident
		} else {
			panic!("First input must be an identifier");
		}
	} else {
		panic!("First argument of function must be a single identifier");
	};
	let out = quote! {
		#(#attrs)*
		#vis #constness #asyncness #unsafety #abi #fn_token #ident #generics ( #inputs ) #variadic #output {
			let mut __data = #first_input_ident.data.write().await;
			let __counter = __data.get_mut::<ebina_types::CommandCounter>().expect("Expected CommandCounter in TypeMap");
			let __entry = __counter.entry(#name.to_string()).or_insert(0);
			*__entry += 1;
			#block
		}
	};

	TokenStream::from(out)
}
