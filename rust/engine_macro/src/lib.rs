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
    let counter2 = iter::successors(Some(0u32), |x| Some(x + 1));

    quote!(
        #[derive(Default, Clone)]
        #[derive(Serialize, Deserialize)]
        pub struct ComponentStorage {
            #( #component_storages ),*
        }
        #(
            impl ::engine_ecs::LocalTypeIndex<ComponentStorage> for #component_types {
                const TYPE_INDEX: u32 = #counter;
            }

            impl ::engine_ecs::internal::ComponentStorageProvider<#component_types> for ComponentStorage {
                fn storage(&self) -> & ::engine_ecs::internal::ComponentList<#component_types> {
                    & self.#component_storage_names
                }
                fn storage_mut(&mut self) -> &mut ::engine_ecs::internal::ComponentList<#component_types> {
                    &mut self.#component_storage_names
                }
            }
        )*

        impl ::engine_ecs::internal::DynDispath for ComponentStorage {
            fn dispath_mut<F, Ret>(&mut self, index: ::engine_ecs::TypeIndex, f: F) -> Ret
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

        #(
            impl ::engine_ecs::Bundle<ComponentStorage> for #component_types {
                fn type_ids() -> ::engine_ecs::internal::TypeIndexStorage {
                    ::engine_ecs::internal::TypeIndexStorage::from_elem(#counter2, 1)
                }

                fn add_to_archetype_in_storage(self, world: &mut World<ComponentStorage>, archetype: ::engine_ecs::ArchetypeID) {
                    world.add_bundle_to_archetype(archetype, self)
                }
            }
        )*
    )
    .into()
}

/// Internal use
#[proc_macro]
pub fn gen_bundle_tuple_impls(input: TokenStream) -> TokenStream {
    let count = input
        .into_iter()
        .next()
        .unwrap()
        .to_string()
        .parse::<usize>()
        .unwrap();

    let type_names = (0..count)
        .map(|x| format_ident!("B{x}"))
        .collect::<Vec<_>>();
    let type_names_from_1 = type_names.iter().skip(1);
    let counter = (0..count).map(syn::Index::from);

    quote!(
        impl<Storage, #(#type_names,)*> Bundle<Storage> for (#(#type_names,)*)
        where
            #(#type_names: Bundle<Storage>),*
        {
            fn type_ids() -> TypeIndexStorage {
                let mut indexes = B0::type_ids();
                #(indexes.extend_from_slice(& #type_names_from_1 ::type_ids());)*
                indexes
            }

            fn add_to_archetype_in_storage(
                self,
                world: &mut World<Storage>,
                archetype: ArchetypeID,
            ) {
                #(self.#counter.add_to_archetype_in_storage(world, archetype);)*
            }
        }
    )
    .into()
}
