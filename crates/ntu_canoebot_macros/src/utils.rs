use std::borrow::BorrowMut;

/// Substitute one token tree into a wildcard in another token tree
/// This is working
pub fn substitute_into_wildcard(
    target: &proc_macro2::TokenStream,
    sub: &proc_macro2::TokenStream,
    delimiter: proc_macro2::Delimiter,
) -> proc_macro2::TokenStream {
    let new_ast = target.clone();

    let mut delim = delimiter;

    let new_stream = new_ast
        .clone()
        .into_iter()
        .borrow_mut()
        .map(|mut item| match &mut item {
            proc_macro2::TokenTree::Group(g) => {
                delim = g.delimiter();

                let stream = g.stream();

                let substituted = match has_wildcard_token(&stream) {
                    true => substitute_into_wildcard(&stream, sub, delim),
                    false => return item,
                };

                // let substituted = if has_wildcard_token(&stream) {
                //     substitute_into_wildcard(&stream, sub, delim)
                // } else {
                //     // do nothing
                //     return item;
                // };

                let new =
                    proc_macro2::TokenTree::Group(proc_macro2::Group::new(delim, substituted));

                new
            }

            proc_macro2::TokenTree::Ident(i) => match i == "_" {
                true => {
                    let new = proc_macro2::TokenTree::Group(proc_macro2::Group::new(
                        proc_macro2::Delimiter::None,
                        sub.clone(),
                    ));

                    new
                }
                false => item,
            },

            _ => item,
        })
        .collect::<proc_macro2::TokenStream>();

    new_stream
}

/// Check if a token tree has a wildcard (at the top level)
pub fn has_wildcard_token(ts: &proc_macro2::TokenStream) -> bool {
    ts.clone().into_iter().any(|tree| {
        match tree {
            // check that the group delimiter is parenthesis
            proc_macro2::TokenTree::Group(g) => {
                // match g.delimiter() {
                //     proc_macro2::Delimiter::Parenthesis => (),
                //     _ => return false,
                // }
                let stream = g.stream();
                // false
                has_wildcard_token(&stream)
            }
            proc_macro2::TokenTree::Ident(i) => i.eq("_"),
            // false},
            _ => false,
        }
    })
}
