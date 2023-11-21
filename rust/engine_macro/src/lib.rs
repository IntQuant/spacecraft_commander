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
    let counter3 = iter::successors(Some(0u32), |x| Some(x + 1));

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

            unsafe impl<'a> ::engine_ecs::internal::QueryParameter<'a, ComponentStorage> for &'a #component_types {
                fn add_requests(req: &mut ::engine_ecs::internal::ComponentRequests) {
                    req.request(#counter3, false);
                    req.require(#counter3);
                }
                unsafe fn get_from_world(
                    world: &'a ::engine_ecs::QueryWorld<'a, ComponentStorage>,
                    archetype: ::engine_ecs::ArchetypeID,
                    index: ::engine_ecs::internal::InArchetypeId,
                    _ent_id: ::engine_ecs::EntityID,
                ) -> Self {
                    let storage = world.storage_for_archetype::<#component_types>(archetype).expect("component assumed to exist, as we've asked for it");
                    world.get(storage, index).expect("component assumed to exist, as it exists in the archetype")
                }
            }

            unsafe impl<'a> ::engine_ecs::internal::QueryParameter<'a, ComponentStorage> for &'a mut #component_types {
                fn add_requests(req: &mut ::engine_ecs::internal::ComponentRequests) {
                    req.request(#counter3, true);
                    req.require(#counter3);
                }
                unsafe fn get_from_world(
                    world: &'a ::engine_ecs::QueryWorld<'a, ComponentStorage>,
                    archetype: ::engine_ecs::ArchetypeID,
                    index: ::engine_ecs::internal::InArchetypeId,
                    _ent_id: ::engine_ecs::EntityID,
                ) -> Self {
                    let storage = world.storage_for_archetype::<#component_types>(archetype).expect("component assumed to exist, as we've asked for it");
                    world.get_mut(storage, index).expect("component assumed to exist, as it exists in the archetype")
                }
            }
        )*

        pub type Query<'a, T, Limits=()> = ::engine_ecs::QueryG<'a, ComponentStorage, T, Limits>;
    )
    .into()
}

/// ecs internal use
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

/// ecs internal use
#[proc_macro]
pub fn gen_query_param_tuple_impls(input: TokenStream) -> TokenStream {
    let count = input
        .into_iter()
        .next()
        .unwrap()
        .to_string()
        .parse::<usize>()
        .unwrap();

    let type_names = (0..count)
        .map(|x| format_ident!("P{x}"))
        .collect::<Vec<_>>();

    quote!(
        unsafe impl<'a, Storage, #(#type_names: QueryParameter<'a, Storage>,)*> QueryParameter<'a, Storage> for (#(#type_names,)*)
        {
            fn add_requests(req: &mut ComponentRequests) {
                #(#type_names::add_requests(req);)*
            }

            unsafe fn get_from_world(
                world: &'a QueryWorld<'a, Storage>,
                archetype: ArchetypeID,
                index: InArchetypeId,
                ent_id: EntityID,
            ) -> Self {
                (#(#type_names::get_from_world(world, archetype, index, ent_id),)*)
            }
        }
    ).into()
}
