use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Token};

use crate::expr::TestCaseExpression;

#[derive(Debug)]
pub struct TestCaseComment {
    _semicolon: Token![;],
    pub comment: LitStr,
    pub expression: Option<TestCaseExpression>,
}

impl Parse for TestCaseComment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _semicolon: input.parse()?,
            comment: input.parse()?,
            expression: input.parse().ok(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::comment::TestCaseComment;
    use proc_macro2::TokenStream;
    use syn::parse_quote;

    #[test]
    fn parses_token_stream() {
        let input: TokenStream = parse_quote! { ; "abcdef" };
        let actual: TestCaseComment = syn::parse2(input).unwrap();
        assert_eq!(actual.comment.value(), "abcdef")
    }
}
