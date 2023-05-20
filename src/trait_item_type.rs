use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    *,
};

pub struct ImplItemBound {
    pub attrs: Vec<Attribute>,
    // pub vis: Visibility,
    // pub defaultness: Option<Token![default]>,
    pub type_token: Token![type],
    pub ident: Ident,
    pub generics: Generics,
    pub colon_token: Token![:],
    pub bounds: Punctuated<TypeParamBound, Token![+]>,
    pub eq_token: Token![=],
    pub expr: Expr,
    pub semi_token: Token![;],
}

impl Parse for ImplItemBound {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let type_token: Token![type] = input.parse()?;
        let ident: Ident = input.parse()?;
        let mut generics: Generics = input.parse()?;
        let (colon_token, bounds) = parse_optional_bounds(input)?;
        let eq_token: Token![=] = input.parse()?;
        let expr: Expr = input.parse()?;
        generics.where_clause = input.parse()?;
        let semi_token: Token![;] = input.parse()?;
        Ok(ImplItemBound {
            attrs,
            type_token,
            ident,
            generics,
            colon_token,
            bounds,
            eq_token,
            expr,
            semi_token,
        })
    }
}

fn parse_optional_bounds(
    input: ParseStream,
) -> Result<(Token![:], Punctuated<TypeParamBound, Token![+]>)> {
    let colon_token: Token![:] = input.parse()?;

    let mut bounds = Punctuated::new();
    loop {
        if input.peek(Token![where]) || input.peek(Token![=]) || input.peek(Token![;]) {
            break;
        }
        bounds.push_value(input.parse::<TypeParamBound>()?);
        if input.peek(Token![where]) || input.peek(Token![=]) || input.peek(Token![;]) {
            break;
        }
        bounds.push_punct(input.parse::<Token![+]>()?);
    }

    Ok((colon_token, bounds))
}
