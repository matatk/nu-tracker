mod make_returned_issue;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn make_returned_issue(_attribute: TokenStream, input: TokenStream) -> TokenStream {
	make_returned_issue::make_returned_issue(input)
}
