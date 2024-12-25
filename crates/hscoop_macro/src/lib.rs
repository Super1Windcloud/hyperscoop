use proc_macro::TokenStream; // 词法树
use quote::quote;
use syn;
use syn::{Data, DeriveInput};

#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    eprintln!("route attr:{:#?}", attr);
    eprintln!("route item:{:#?}", item);
    item
}

#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_hello_macro(&ast)
}

fn impl_hello_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl HelloMacro for #name {
            fn hello_macro() {
                println!("Hello, Macro!   My name is {}!", stringify!(#name));
            }
        }
    };
    gen.into()
}

#[proc_macro]
pub fn make_answer(item: TokenStream) -> TokenStream {
    eprintln!("make_answer:{:#?}", item); //可以通过eprintln!进行打印调试
    "fn answer() -> u32 { 45 }".parse().unwrap()
}

#[proc_macro_derive(IntoHashMap)]
pub fn into_hash_map(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    let struct_identifier = &input.ident;

    match &input.data {
    Data::Struct(syn::DataStruct { fields, .. }) => {
      let mut implementation = quote! {
                let mut hash_map = std::collections::HashMap::<String, String>::new();
            };

      for field in fields {
        let identifier = field.ident.as_ref().unwrap();
        implementation.extend(quote! {
                    hash_map.insert(stringify!(#identifier).to_string(), String::from(value.#identifier));
                });
      }

      quote! {
                #[automatically_derived]
                impl From<#struct_identifier> for std::collections::HashMap<String, String> {
                    fn from(value: #struct_identifier) -> Self {
                        #implementation

                        hash_map
                    }
                }
            }
    }
    _ => unimplemented!()
  }.into()
}

#[proc_macro_derive(MyDefault)]
pub fn my_default(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let id = ast.ident;

    let Data::Struct(s) = ast.data else {
        panic!("MyDefault derive macro must use in struct");
    };

    // 声明一个新的ast，用于动态构建字段赋值的token
    let mut field_ast = quote!();

    // 这里就是要动态添加token的地方了，需要动态完成Self的字段赋值
    for (idx, f) in s.fields.iter().enumerate() {
        let (field_id, field_ty) = (&f.ident, &f.ty);

        if field_id.is_none() {
            let field_idx = syn::Index::from(idx);
            field_ast.extend(quote! {});
        } else {
            field_ast.extend(quote! {});
        }
    }

    let result = quote! {
        impl std::default::Default for #id {
            fn default() -> Self {
                Self { #field_ast }
            }
        }
    }
    .into();
    result
}
