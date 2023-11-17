use std::iter;

use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[proc_macro]
pub fn gen_storage_for_world(input: TokenStream) -> TokenStream {
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
        .map(|(c_name, c_storage)| quote!(#c_storage : ::engine_ecs::internal::ComponentList<#c_name>));

    let counter = iter::successors(Some(0u32), |x| Some(x + 1));

    quote!(
        pub struct ComponentStorage {
            #( #component_storages ),*
        }
        #(
            impl ::engine_ecs::LocalTypeIndex<ComponentStorage> for #component_types {
                const TYPE_INDEX: u32 = #counter;
            }

            impl ::engine_ecs::internal::ComponentStorageProvider<#component_types> for ComponentStorage {
                fn storage_mut(&mut self) -> &mut ::engine_ecs::internal::ComponentList<#component_types> {
                    &mut self.#component_storage_names
                }
            }
        )*

        impl ::engine_ecs::internal::DynDispath for ComponentStorage {
            fn dispath_mut<F, Ret, T>(&mut self, index: ::engine_ecs::TypeIndex, f: F) -> Ret
            where
                F: FnOnce(&mut dyn ::engine_ecs::internal::DynComponentList) -> Ret
            {
                match index {
                    #(
                        <#component_types as ::engine_ecs::LocalTypeIndex<ComponentStorage>>::TYPE_INDEX => {
                            let storage = <Self as ::engine_ecs::internal::ComponentStorageProvider<#component_types>>::storage_mut(self);
                            f(storage)
                        }
                    ,)*
                    _ => unreachable!()
                }
            }
        }
    )
    .into()
}
