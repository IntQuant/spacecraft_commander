use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[proc_macro]
pub fn gen_world(input: TokenStream) -> TokenStream {
    let mut component_names = Vec::new();
    for token in input {
        component_names.push(token.to_string());
    }

    let component_storage_names = component_names
        .iter()
        .map(|c| format_ident!("component_storage_{}", c.to_lowercase()))
        .collect::<Vec<_>>();

    let component_types = component_names
        .iter()
        .map(|c| format_ident!("{c}"))
        .collect::<Vec<_>>();

    let component_storages = component_types
        .iter()
        .zip(component_storage_names.iter())
        .map(|(c_name, c_storage)| quote!(#c_storage : Vec<Vec<#c_name>>));

    quote!(
        pub struct ComponentStorage {
            #( #component_storages ),*
        }
        #(
            impl ::engine_ecs::internal::ComponentStorageProvider<#component_types> for ComponentStorage {
                fn storage_for(&mut self) -> &mut Vec<Vec<#component_types>> {
                    &mut self.#component_storage_names
                }
            }
        )*
    )
    .into()
}
