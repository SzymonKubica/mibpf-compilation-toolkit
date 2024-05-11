extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, LitInt, LitStr, Expr, Token, parse::Parse};

struct MacroInput {
    pub env_name: Expr,
    pub comma: Token![,],
    pub env_val: Expr,
}

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            env_name: input.parse()?,
            comma: input.parse()?,
            env_val: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn set_env_or_default(_input: TokenStream) -> TokenStream {

    let input = parse_macro_input!(_input as MacroInput);

    let env_name = match input.env_name {
        Expr::Lit(lit) => {
            match lit.lit {
                syn::Lit::Str(lit_str) => {
                    lit_str.value()
                }
                _ => panic!("Expected a string literal")
            }
        }
        _ => panic!("Expected a string literal")
    };

    let mut env_value = match input.env_val {
        Expr::Lit(lit) => {
            match lit.lit {
                syn::Lit::Int(lit_str) => {
                    lit_str.base10_parse::<usize>().unwrap()
                }
                _ => panic!("Expected a string literal")
            }
        }
        _ => panic!("Expected a string literal")
    };

    // option_env! takes a string literal so we need to conditionally match
    // here over the env vars that we want to support.
    let maybe_env_var = match env_name.as_str() {
        "SUIT_STORAGE_SLOTS" => { option_env!("SUIT_STORAGE_SLOTS") },
        "SUIT_STORAGE_SLOT_SIZE" => { option_env!("SUIT_STORAGE_SLOT_SIZE") },
        _ => None,
    };

    env_value = if let Some(env_value) = maybe_env_var {
        env_value.parse().unwrap()
    } else {
        env_value
    };

    let output = format!("{}", env_value);
    output.parse().unwrap()
}

