use proc_macro::TokenStream;

use quote::quote;
use syn::{
	parse, visit::Visit, Block, Expr, ExprField, FnArg, Ident, ImplItem, Item, ItemFn, ItemImpl,
	Member, Pat, Signature, Type,
};

struct ExprFieldVisitor {
	target_ident: Ident,
	found_fields: Vec<Ident>,
}

impl ExprFieldVisitor {
	const fn new(target_ident: Ident) -> Self {
		Self {
			target_ident,
			found_fields: vec![],
		}
	}
}

impl<'ast> Visit<'ast> for ExprFieldVisitor {
	fn visit_expr_field(&mut self, e: &'ast ExprField) {
		if let Expr::Path(sp) = &*e.base {
			if sp.path.segments.len() == 1
				&& sp.path.segments.first().unwrap().ident == self.target_ident
			{
				if let Member::Named(name) = &e.member {
					self.found_fields.push(name.clone());
				}
			}
		}

		syn::visit::visit_expr_field(self, e);
	}
}

fn type_for_field_named(name: &str) -> proc_macro2::TokenStream {
	match name {
		"assignees" => quote! { Vec<crate::returned_issue::Assignee> },
		"number" => quote! { u32 },
		"title" | "body" => quote! { String },
		"repository" => quote! { crate::returned_issue::Repository },
		"labels" => quote! { Vec<crate::returned_issue::Label> },
		"author" => quote! { crate::returned_issue::Assignee },
		_ => panic!("unexpected issue field name"),
	}
}

pub fn make_returned_issue(input: TokenStream) -> TokenStream {
	match parse(input).unwrap() {
		Item::Impl(thing) => process_impl(&thing),
		Item::Fn(thing) => process_fn(&thing),
		_ => panic!("expected impl or fn"),
	}
}

fn process_impl(implementation: &ItemImpl) -> TokenStream {
	let ImplItem::Fn(func) = &implementation.items[0] else {
		panic!("expected fn as first impl member")
	};

	let struct_stuff = create_struct(&func.sig, &func.block);

	quote! {
		#struct_stuff
		#implementation
	}
	.into()
}

fn process_fn(func: &ItemFn) -> TokenStream {
	let struct_stuff = create_struct(&func.sig, &func.block);

	quote! {
		#struct_stuff
		#func
	}
	.into()
}

fn create_struct(sig: &Signature, blok: &Block) -> proc_macro2::TokenStream {
	let first_arg = sig.inputs.first().unwrap();

	let FnArg::Typed(first_arg_typed) = first_arg else {
		panic!("expected first argument to be typed")
	};

	let var_name = match *first_arg_typed.pat {
		Pat::Ident(ref name) => name.ident.clone(),
		_ => panic!("expected first argument's name to be an ident"),
	};

	let var_type = match *first_arg_typed.ty {
		Type::Path(ref tp) => tp.path.segments.first().unwrap().ident.clone(),
		_ => panic!("expected first argument's type to be a path"),
	};

	let mut efv = ExprFieldVisitor::new(var_name);
	efv.visit_block(blok);

	let struct_field_names = efv
		.found_fields
		.iter()
		.map(|name| {
			let n = name.to_string();
			quote! { #n }
		})
		.collect::<Vec<_>>();

	let struct_fields = efv.found_fields.iter().map(|name| {
		let field_type = type_for_field_named(&name.to_string());
		quote! { #name: #field_type }
	});

	quote! {
		#[derive(::serde::Deserialize)]
		struct #var_type {
			 #(#struct_fields),*
		}

		impl ReturnedIssue for #var_type {
			const GITHUB_FIELD_NAMES: &'static [&'static str] = &[#(#struct_field_names),*];
		}
	}
}
