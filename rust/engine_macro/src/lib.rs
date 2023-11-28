use std::iter;

use proc_macro::TokenStream;
use quote::{format_ident, quote};

enum GenState {
    None,
    Components,
    Resources,
}

#[proc_macro]
pub fn gen_storage_for_world(input: TokenStream) -> TokenStream {
    let mut gen_state = GenState::None;
    let mut component_names = Vec::new();
    let mut resource_names = Vec::new();

    let mut input_iter = input.into_iter();
    while let Some(token_raw) = input_iter.next() {
        let token_str = token_raw.to_string();
        match token_str.as_str() {
            ":" => {
                let new_mode = input_iter.next().map(|x| x.to_string()).unwrap_or_default();
                match new_mode.as_str() {
                    "components" => gen_state = GenState::Components,
                    "resources" => gen_state = GenState::Resources,
                    _ => panic!("Unknown requested state: {}", new_mode),
                }
            }
            _ => {
                match gen_state {
                    GenState::None => panic!("Set current mode with ': <mode>', where <mode> is one of 'components', 'resources'"),
                    GenState::Components => component_names.push(token_str),
                    GenState::Resources => resource_names.push(token_str),
                }
            }
        }
    }
    // for token in input {
    //     component_names.push(token.to_string());
    // }

    let component_storage_names = component_names
        .iter()
        .enumerate()
        .map(|(i, _c)| format_ident!("component_storage_{}", i))
        .collect::<Vec<_>>();

    let resource_storage_names = resource_names
        .iter()
        .enumerate()
        .map(|(i, _c)| format_ident!("resource_storage_{}", i))
        .collect::<Vec<_>>();

    let component_types = component_names
        .iter()
        .map(|c| format_ident!("{c}"))
        .collect::<Vec<_>>();

    let resource_types = resource_names
        .iter()
        .map(|c| format_ident!("{}", c))
        .collect::<Vec<_>>();

    let component_storages = component_types
        .iter()
        .zip(component_storage_names.iter())
        .map(|(c_name, c_storage)| quote!(#c_storage : ::engine_ecs::internal::ComponentList<#c_name>));

    let counter = iter::successors(Some(0u32), |x| Some(x + 1));
    let counter2 = iter::successors(Some(0u32), |x| Some(x + 1));
    let counter3 = iter::successors(Some(0u32), |x| Some(x + 1));

    let counter_resources = 0..(resource_names.len() as u32);
    let counter_resources_2 = 0..(resource_names.len() as u32);

    quote!(
        #[derive(Default, Clone)]
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        pub struct ComponentStorage {
            #( #component_storages ,)*
            #(#resource_storage_names: ::engine_ecs::internal::ResourceStorage<#resource_types>,)*
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

        #(
            impl ::engine_ecs::LocalTypeIndex<::engine_ecs::internal::OfResources<ComponentStorage>> for #component_types {
                const TYPE_INDEX: u32 = #counter_resources;
            }

            impl ::engine_ecs::internal::ResourceStorageProvider<#resource_types> for ComponentStorage {
                fn storage(&self) -> & ::engine_ecs::internal::ResourceStorage<#resource_types> {
                    & self.#resource_storage_names
                }
                fn storage_mut(&mut self) -> &mut ::engine_ecs::internal::ResourceStorage<#resource_types> {
                    &mut self.#resource_storage_names
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

                fn add_to_archetype_in_storage(self, world: &mut ::engine_ecs::World<ComponentStorage>, archetype: ::engine_ecs::ArchetypeID) {
                    world.add_bundle_to_archetype(archetype, self)
                }
            }

            unsafe impl<'wrld> ::engine_ecs::internal::QueryParameter<'wrld, ComponentStorage> for &'wrld #component_types {
                fn add_requests(req: &mut ::engine_ecs::internal::ComponentRequests) {
                    req.request(#counter3, false);
                    req.require(#counter3);
                }
                unsafe fn get_from_world(
                    world: &'wrld ::engine_ecs::QueryWorld<'wrld, ComponentStorage>,
                    archetype: ::engine_ecs::ArchetypeID,
                    index: ::engine_ecs::internal::InArchetypeID,
                    _ent_id: ::engine_ecs::EntityID,
                ) -> Self {
                    let storage = world.storage_for_archetype::<#component_types>(archetype).expect("component assumed to exist, as we've asked for it");
                    world.get(storage, index).expect("component assumed to exist, as it exists in the archetype")
                }
            }

            unsafe impl<'wrld> ::engine_ecs::internal::QueryParameter<'wrld, ComponentStorage> for &'wrld mut #component_types {
                fn add_requests(req: &mut ::engine_ecs::internal::ComponentRequests) {
                    req.request(#counter3, true);
                    req.require(#counter3);
                }
                unsafe fn get_from_world(
                    world: &'wrld ::engine_ecs::QueryWorld<'wrld, ComponentStorage>,
                    archetype: ::engine_ecs::ArchetypeID,
                    index: ::engine_ecs::internal::InArchetypeID,
                    _ent_id: ::engine_ecs::EntityID,
                ) -> Self {
                    let storage = world.storage_for_archetype::<#component_types>(archetype).expect("component assumed to exist, as we've asked for it");
                    world.get_mut(storage, index).expect("component assumed to exist, as it exists in the archetype")
                }
            }
        )*

        #(
            unsafe impl<'a> ::engine_ecs::internal::SystemParameter<'a, ComponentStorage> for &'a #resource_types {
                fn requests() -> ::engine_ecs::internal::SmallVec<[::engine_ecs::internal::ComponentRequests; 8]> {
                    let mut ret = ::engine_ecs::internal::SmallVec::default();
                    let mut req = ::engine_ecs::internal::ComponentRequests::default();
                    req.request_resource(#counter_resources_2, false);
                    ret.push(req);
                    ret
                }
                unsafe fn from_world(world: &'a ::engine_ecs::QueryWorld<'a, ComponentStorage>) -> Self {
                    world.resource()
                }
            }

            unsafe impl<'a> ::engine_ecs::internal::SystemParameter<'a, ComponentStorage> for &'a mut #resource_types {
                fn requests() -> ::engine_ecs::internal::SmallVec<[::engine_ecs::internal::ComponentRequests; 8]> {
                    let mut ret = ::engine_ecs::internal::SmallVec::default();
                    let mut req = ::engine_ecs::internal::ComponentRequests::default();
                    req.request_resource(#counter_resources_2, true);
                    ret.push(req);
                    ret
                }
                unsafe fn from_world(world: &'a ::engine_ecs::QueryWorld<'a, ComponentStorage>) -> Self {
                    world.resource_mut()
                }
            }
        )*

        pub type Query<'a, T, Limits=()> = ::engine_ecs::QueryG<'a, ComponentStorage, T, Limits>;
        pub type With<T> = ::engine_ecs::WithG<ComponentStorage, T>;
        pub type Without<T> = ::engine_ecs::WithoutG<ComponentStorage, T>;
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
        unsafe impl<'wrld, Storage, #(#type_names: QueryParameter<'wrld, Storage>,)*> QueryParameter<'wrld, Storage> for (#(#type_names,)*)
        {
            fn add_requests(req: &mut ComponentRequests) {
                #(#type_names::add_requests(req);)*
            }

            unsafe fn get_from_world(
                world: &'wrld QueryWorld<'wrld, Storage>,
                archetype: ArchetypeID,
                index: InArchetypeID,
                ent_id: EntityID,
            ) -> Self {
                (#(#type_names::get_from_world(world, archetype, index, ent_id),)*)
            }
        }

        impl<#(#type_names: QueryLimits,)*> QueryLimits for (#(#type_names,)*)
        {
            fn add_requests(req: &mut ComponentRequests) {
                #(#type_names::add_requests(req);)*
            }
        }
    ).into()
}

/// ecs internal use
#[proc_macro]
pub fn gen_fn_system_impls(input: TokenStream) -> TokenStream {
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
    let var_names = (0..count)
        .map(|x| format_ident!("p{x}"))
        .collect::<Vec<_>>();

    quote!(
        impl<'a, Storage, #(#type_names),*> System<'a, Storage> for fn(#(#type_names),*) -> ()
        where
            #(#type_names: SystemParameter<'a, Storage>,)*
        {
            fn requests() -> SmallVec<[ComponentRequests; 8]> {
                let mut ret = SmallVec::default();
                #(ret.extend(#type_names::requests());)*
                ret
            }

            fn run(self, world: QueryWorld<'a, Storage>) {
                #(let #var_names = world.parameter();)*
                self(#(#var_names,)*)
            }
        }

    )
    .into()
}

/// ecs internal use
#[proc_macro]
pub fn gen_world_run_impls(input: TokenStream) -> TokenStream {
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
    let var_names = (0..count)
        .map(|x| format_ident!("p{x}"))
        .collect::<Vec<_>>();

    quote!(
        impl<'wrld, Storage, F, #(#type_names,)*> WorldRun<'wrld, F, (#(#type_names,)*)> for QueryWorld<'wrld, Storage>
        where
            F: FnOnce(#(#type_names),*),
            #(#type_names: SystemParameter<'wrld, Storage>,)*
        {
            fn run(&'wrld self, f: F) {
                #( let #var_names = self.parameter(); )*
                f(#(#var_names),*)
            }
        }
    )
    .into()
}
