use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{self, Parser},
    parse_macro_input, DeriveInput, ItemStruct,
};

#[proc_macro_attribute]
pub fn mrx_cli(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let _ = parse_macro_input!(args as parse::Nothing);
    let field = quote! {
        /// Override the config file path
        #[arg(short, long)]
        config: Option<String>
    };

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        fields
            .named
            .push(syn::Field::parse_named.parse2(field).unwrap());
    }

    quote! {
        #item_struct
    }
    .into()
}

#[proc_macro_derive(MrxCli)]
pub fn mrx_cli_derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let ident = &derive_input.ident;

    quote! {
        type MrxCliArgsResult<T: Sized> = mrx_utils::ConfigInitResult<(mrx_utils::Config, T)>;

        impl mrx_utils::MrxCli for #ident {
            fn create_mrx_cli_args() -> MrxCliArgsResult<Self> {

                let cli = Self::parse();

                use mrx_utils::Config;

                cli.config
                    .clone()
                    .map_or_else(Config::default_init, Config::try_from)
                    .map(|config| (config, cli))
            }
        }

        impl #ident {
            pub fn args() -> MrxCliArgsResult<Self> {
                Self::create_mrx_cli_args()
            }
        }
    }
    .into()
}
