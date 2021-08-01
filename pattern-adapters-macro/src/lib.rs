#![feature(never_type)]

use std::convert::TryFrom;

use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use regex_syntax::ast::parse::Parser;
use syn::Lit;

mod pattern;
mod pattern_kind;

use crate::pattern::Pattern;

#[proc_macro]
pub fn regex_pattern(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);

    dbg!(&input);
    let literal = {
        // TODO: assert exactly one literal in the whole input!
        if let Some(TokenTree::Literal(literal)) = input.into_iter().next() {
            literal
        } else {
            panic!("return some kind of error here");
        }
    };

    let lit_str = {
        if let Lit::Str(lit_str) = syn::Lit::new(literal.clone()) {
            lit_str
        } else {
            panic!("unsupported literal :(");
        }
    };

    // TODO: handle error
    let ast = Parser::new().parse(&lit_str.value()).unwrap();

    let pattern = Pattern::try_from(ast).unwrap();
    dbg!(&quote!(#pattern));
    proc_macro::TokenStream::from(quote!(#pattern))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
