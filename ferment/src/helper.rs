use std::collections::{HashMap, HashSet};
use quote::{quote, ToTokens};
use syn::{AngleBracketedGenericArguments, Attribute, Fields, FieldsNamed, FieldsUnnamed, FnArg, GenericArgument, GenericParam, Generics, Ident, ImplItem, ImplItemConst, ImplItemMethod, ImplItemType, Item, ItemConst, ItemEnum, ItemExternCrate, ItemFn, ItemImpl, ItemMacro, ItemMacro2, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, ItemUse, Meta, NestedMeta, parse_quote, Path, PathArguments, PathSegment, PatType, ReturnType, Signature, TraitBound, TraitItem, TraitItemConst, TraitItemMethod, TraitItemType, Type, TypeArray, TypeParamBound, TypePath, TypeReference, TypeTraitObject, TypeTuple, Variant, WherePredicate};
use syn::__private::{Span, TokenStream2};
use syn::punctuated::Punctuated;
use syn::token::Add;
use crate::composition::{GenericBoundComposition, GenericConversion, ImportComposition, TypeComposition};
use crate::context::TypeChain;
use crate::conversion::{ImportConversion, ItemConversion, MacroAttributes, type_ident, type_ident_ref, TypeConversion};
use crate::ext::NestingExtension;
use crate::formatter::format_token_stream;
use crate::holder::{PathHolder, TypeHolder};
use crate::tree::ScopeTreeExportID;

pub trait ItemExtension {
    fn scope_tree_export_id(&self) -> ScopeTreeExportID;
    fn maybe_attrs(&self) -> Option<&Vec<Attribute>>;
    fn maybe_ident(&self) -> Option<&Ident>;
    fn ident_string(&self) -> String {
        self.maybe_ident().map_or(format!("(None)"), Ident::to_string)
    }
    fn maybe_generics(&self) -> Option<&Generics>;

    fn collect_compositions(&self) -> Vec<TypeHolder>;

    fn classify_imports(&self, imports: &HashMap<PathHolder, Path>) -> HashMap<ImportConversion, HashSet<ImportComposition>>;


    fn maybe_generic_bound_for_path(&self, path: &Path) -> Option<GenericBoundComposition> {
        self.maybe_generics()
            .and_then(|generics| maybe_generic_type_bound(path, generics))
    }

    fn get_used_imports(&self, imports: &HashMap<PathHolder, Path>) -> HashMap<ImportConversion, HashSet<ImportComposition>> {
        self.classify_imports(imports)
            .into_iter()
            .filter_map(|(import_type, used_imports)|
                import_type.get_imports_for(used_imports))
            .collect()
    }

    fn find_generics(&self) -> HashSet<TypeHolder> {
        let compositions = self.collect_compositions();
        // collect all types with generics and ensure their uniqueness
        // since we don't want to implement interface multiple times for same object
        let mut generics: HashSet<TypeHolder> = HashSet::new();
        compositions
            .iter()
            .for_each(|TypeHolder(field_type)|
                collect_generic_types_in_type(field_type, &mut generics));
        generics
    }
    fn find_generics_fq(&self, chain: &TypeChain) -> HashSet<GenericConversion> {
        self.find_generics()
            .iter()
            .filter_map(|ty| chain.get(ty))
            .map(GenericConversion::from)
            .collect()
    }

}



impl ItemExtension for Item {
    fn scope_tree_export_id(&self) -> ScopeTreeExportID {
        match self {
            Item::Mod(ItemMod { ident, .. }, ..) |
            Item::Struct(ItemStruct { ident, .. }, ..) |
            Item::Enum(ItemEnum { ident, .. }, ..) |
            Item::Type(ItemType { ident, .. }, ..) |
            Item::Fn(ItemFn { sig: Signature { ident, .. }, .. }, ..) |
            Item::Trait(ItemTrait { ident, .. }, ..) => ScopeTreeExportID::Ident(ident.clone()),
            Item::Use(ItemUse { .. }, ..) =>
                panic!("Not  supported"),
            Item::Impl(ItemImpl { self_ty, trait_, .. }, ..) => ScopeTreeExportID::Impl(*self_ty.clone(), trait_.clone().map(|(_, path, _)| path)),
            item => panic!("ScopeTreeExportID Not supported for {}", quote!(#item)),
            // type_ident(self_ty).unwrap(),
        }

    }

    fn maybe_attrs(&self) -> Option<&Vec<Attribute>> {
        match self {
            Item::Const(item) => Some(&item.attrs),
            Item::Enum(item) => Some(&item.attrs),
            Item::ExternCrate(item) => Some(&item.attrs),
            Item::Fn(item) => Some(&item.attrs),
            Item::ForeignMod(item) => Some(&item.attrs),
            Item::Impl(item) => Some(&item.attrs),
            Item::Macro(item) => Some(&item.attrs),
            Item::Macro2(item) => Some(&item.attrs),
            Item::Mod(item) => Some(&item.attrs),
            Item::Static(item) => Some(&item.attrs),
            Item::Struct(item) => Some(&item.attrs),
            Item::Trait(item) => Some(&item.attrs),
            Item::TraitAlias(item) => Some(&item.attrs),
            Item::Type(item) => Some(&item.attrs),
            Item::Union(item) => Some(&item.attrs),
            Item::Use(item) => Some(&item.attrs),
            _ => None,
        }
    }

    fn maybe_ident(&self) -> Option<&Ident> {
        match self {
            Item::Const(ItemConst { ident, .. }) |
            Item::Enum(ItemEnum { ident, .. }) |
            Item::ExternCrate(ItemExternCrate { ident, .. }) |
            Item::Fn(ItemFn { sig: Signature { ident, .. }, .. }) |
            Item::Macro2(ItemMacro2 { ident, .. }) |
            Item::Mod(ItemMod { ident, .. }) |
            Item::Struct(ItemStruct { ident, ..  }) |
            Item::Static(ItemStatic { ident, ..  }) |
            Item::Trait(ItemTrait { ident, ..  }) |
            Item::TraitAlias(ItemTraitAlias { ident, ..  }) |
            Item::Type(ItemType { ident, .. }) |
            Item::Union(ItemUnion { ident, .. }) => Some(ident),
            Item::Macro(ItemMacro { ident, .. }) => ident.as_ref(),
            Item::Impl(ItemImpl { self_ty, .. }) => type_ident_ref(self_ty),
            _ => None
        }
    }
    fn maybe_generics(&self) -> Option<&Generics> {
        match self {
            Item::Enum(ItemEnum { generics, .. }) |
            Item::Fn(ItemFn { sig: Signature { generics, .. }, .. }) |
            Item::Impl(ItemImpl { generics, .. }) |
            Item::Struct(ItemStruct { generics, .. }) |
            Item::Trait(ItemTrait { generics, .. }) |
            Item::TraitAlias(ItemTraitAlias { generics, .. }) |
            Item::Type(ItemType { generics, .. }) |
            Item::Union(ItemUnion { generics, .. }) =>
                Some(generics),
            _ => None
        }
    }

    fn collect_compositions(&self) -> Vec<TypeHolder> {
        let mut type_and_paths: Vec<TypeHolder> = Vec::new();
        let mut cache_type = |ty: &Type|
            type_and_paths.push(TypeHolder(ty.clone()));
        let mut cache_fields = |fields: &Fields, _attrs: &MacroAttributes| match fields {
            Fields::Unnamed(FieldsUnnamed { unnamed: fields, .. }) |
            Fields::Named(FieldsNamed { named: fields, .. }) =>
                fields.iter().for_each(|field| cache_type(&field.ty)),
            Fields::Unit => {}
        };
        // let mut cache_sig = |ref sig: &Signature| {
        // };
        match self {
            Item::Mod(ItemMod { content: Some((_, items)), .. }) =>
                items.iter()
                    // .flat_map(|m| m.collect_composition()))
                    .for_each(|item|
                        type_and_paths.extend(item.collect_compositions())),
            Item::Struct(item_struct, ..) =>
                handle_attributes_with_handler(&item_struct.attrs, |attrs|
                    cache_fields(&item_struct.fields, &attrs)),
            Item::Enum(item_enum, ..) =>
                handle_attributes_with_handler(&item_enum.attrs, |attrs|
                    item_enum.variants.iter().for_each(|Variant { fields, .. }|
                        cache_fields(fields, &attrs))),
            Item::Type(ItemType { attrs, ty, .. }, ..) =>
                handle_attributes_with_handler(attrs, |_attrs|
                    cache_type(ty)),
            Item::Fn(item_fn, ..) =>
                handle_attributes_with_handler(&item_fn.attrs, |_attrs| {
                    type_and_paths.extend(item_fn.sig.collect_compositions());
                }),
            Item::Impl(item_impl) => handle_attributes_with_handler(&item_impl.attrs, |_attrs| {
                item_impl.items.iter().for_each(|impl_item| match impl_item {
                    ImplItem::Const(ImplItemConst { ty, .. }) =>
                        cache_type(ty),
                    ImplItem::Method(ImplItemMethod { sig, .. }) => {
                        sig.inputs.iter().for_each(|arg|
                            if let FnArg::Typed(PatType { ty, .. }) = arg {
                                cache_type(ty);
                            });
                        if let ReturnType::Type(_, ty) = &sig.output {
                            cache_type(ty);
                        }
                    },
                    ImplItem::Type(ImplItemType { ty, .. }) =>
                        cache_type(ty),
                    _ => {}
                });
            }),
            Item::Trait(item_trait, ..) => handle_attributes_with_handler(&item_trait.attrs, |_attrs| {
                item_trait.items.iter().for_each(|trait_item| match trait_item {
                    TraitItem::Type(TraitItemType { default: Some((_, ty)), .. }) =>
                        cache_type(ty),
                    TraitItem::Method(TraitItemMethod { sig, .. }) => {
                        sig.inputs.iter().for_each(|arg|
                            if let FnArg::Typed(PatType { ty, .. }) = arg {
                                cache_type(ty);
                            });
                        if let ReturnType::Type(_, ty) = &sig.output {
                            cache_type(ty);
                        }
                    },
                    TraitItem::Const(TraitItemConst { ty, .. }) =>
                        cache_type(ty),
                    _ => {}
                });
            }),
            _ => {}
        }

        type_and_paths
    }

    fn classify_imports(&self, imports: &HashMap<PathHolder, Path>) -> HashMap<ImportConversion, HashSet<ImportComposition>> {
        let mut container = HashMap::new();
        match self {
            Item::Mod(ItemMod { content: Some((_, items)), .. }) =>
                items.iter()
                    .for_each(|item|
                        container.extend(item.classify_imports(imports))),
            Item::Struct(item_struct) =>
                handle_attributes_with_handler(&item_struct.attrs, |_path|
                    cache_fields_in(&mut container, &item_struct.fields, imports)),
            Item::Enum(item_enum) =>
                handle_attributes_with_handler(&item_enum.attrs, |_path| item_enum.variants.iter().for_each(|Variant { fields, .. }|
                    cache_fields_in(&mut container, fields, imports))),
            Item::Type(ItemType { attrs, ty, .. }, .. ) =>
                handle_attributes_with_handler(attrs, |_path|
                    cache_type_in(&mut container, ty, imports)),
            Item::Fn(item_fn, ..) =>
                handle_attributes_with_handler(&item_fn.attrs, |_path|
                    container.extend(item_fn.sig.classify_imports(imports))
                ),
            Item::Trait(item_trait, ..) =>
                handle_attributes_with_handler(&item_trait.attrs, |_path| {
                    item_trait.items.iter().for_each(|trait_item| match trait_item {
                        TraitItem::Method(TraitItemMethod { sig, .. }) => {
                            sig.inputs.iter().for_each(|arg| {
                                if let FnArg::Typed(PatType { ty, .. }) = arg {
                                    cache_type_in(&mut container, ty, imports)
                                }
                            });
                            if let ReturnType::Type(_, ty) = &sig.output {
                                cache_type_in(&mut container, ty, imports)
                            };
                        },
                        TraitItem::Type(TraitItemType { default: Some((_, ty)), .. }) =>
                            cache_type_in(&mut container, ty, imports),
                        _ => {}
                    });
                }),
            _ => {}
        }
        container
    }
}


fn generic_trait_bounds(ty: &Path, ident_path: &Path, bounds: &Punctuated<TypeParamBound, Add>) -> Vec<Path> {
    // println!("generic_trait_bounds.1: {} :: {} :: {}", format_token_stream(ty), format_token_stream(ident_path), format_token_stream(bounds));
    let mut has_bound = false;
    let involved = bounds.iter().filter_map(|b| {
        // println!("generic_trait_bounds.2: {}", quote!(#b));
        match b {
            TypeParamBound::Trait(TraitBound { path, .. }) => {
                //println!("generic_trait_bounds: [{}] {} == {} {}", ident_path.eq(ty), format_token_stream(ty), format_token_stream(path), format_token_stream(bounds));
                let has = ident_path.eq(ty);
                if !has_bound && has {
                    has_bound = true;
                }
                has
                    .then(|| path.clone())
            },
            TypeParamBound::Lifetime(_) => None
        }
    }).collect::<Vec<_>>();
    // if !involved.is_empty() {
        // println!("generic_trait_bounds.3: (result) {}", format_path_vec(&involved));
    // }
    involved
}

fn maybe_generic_type_bound(path: &Path, generics: &Generics) -> Option<GenericBoundComposition> {
    // println!("maybe_generic_type_bound.1: {} in [{} .... {}]", format_token_stream(path), format_token_stream(&generics.params), format_token_stream(&generics.where_clause));
    let result = generics.params.iter().find_map(|param| if let GenericParam::Type(type_param) = param {
        let ty: Type = parse_quote!(#path);
        let ident = &type_param.ident;
        let ident_path: Path = parse_quote!(#ident);
        let has_bounds = ident_path.eq(path);
        let bounds = generic_trait_bounds(path, &ident_path, &type_param.bounds);
        // println!("maybe_generic_type_bound.2: [{}: {}] --> [{}]", has_bounds, quote!(#type_param), format_path_vec(&bounds));
        // println!("maybe_generic_type_bound: (bounds) {} ", format_path_vec(&bounds));
        has_bounds
            .then(|| GenericBoundComposition {
                bounds,
                predicates: generics.where_clause
                    .as_ref()
                    .map(|where_clause|
                        where_clause.predicates
                            .iter()
                            .filter_map(|predicate| match predicate {
                                WherePredicate::Type(predicate_type) => {
                                    // println!("maybe_generic_type_bound:::predicate: [{}] {} ::: {}", ty.eq(&predicate_type.bounded_ty), format_token_stream(predicate_type), format_token_stream(path));
                                    let bounded_ty = &predicate_type.bounded_ty;
                                    let ident_path: Path = parse_quote!(#bounded_ty);
                                    ty.eq(&predicate_type.bounded_ty)
                                        .then(||(
                                            predicate_type.bounded_ty.clone(),
                                            {
                                                let bounds = generic_trait_bounds(&path, &ident_path, &predicate_type.bounds);
                                                // println!("maybe_generic_type_bound.3.... {}: {}: [{}]", format_token_stream(&ident_path), format_token_stream(&predicate_type.bounded_ty), format_path_vec(&bounds));
                                                bounds
                                            }))
                                },
                                _ => None })
                            .collect())
                    .unwrap_or_default(),
                type_composition: TypeComposition::new(ty, Some(generics.clone())),
            })
    } else { None });
    // println!("maybe_generic_type_bound (result): {}", result.as_ref().map_or(format!("None"), |r| r.to_string()));
    result
}

pub fn path_arguments_to_types(arguments: &PathArguments) -> Vec<&Type> {
    match arguments {
        PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =>
            args.iter().filter_map(|arg| match arg {
                GenericArgument::Type(ty) => Some(ty),
                _ => None
            }).collect(),
        _ => Vec::new(),
    }
}

fn path_from_type(ty: &Type) -> Option<&Path> {
    match ty {
        Type::Array(TypeArray { elem, len: _, .. }) => path_from_type(elem),
        Type::Path(TypePath { path, .. }) => Some(path),
        Type::Tuple(TypeTuple { elems, .. }) => {
            let first = elems.first().unwrap();
            path_from_type(first)
            // parse_quote!(#elems)
        }
        _ => None,
    }
}

pub fn path_arguments_to_paths(arguments: &PathArguments) -> Vec<&Path> {
    match arguments {
        PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =>
            args.iter().filter_map(|arg| match arg {
                GenericArgument::Type(ty) => path_from_type(ty),
                // GenericArgument::Type(Type::Reference(TypeReference { mutability, elem })) => Some(path),
                _ => None
            }).collect(),
        _ => Vec::new(),
    }
}

pub fn path_arguments_to_type_conversions(arguments: &PathArguments) -> Vec<TypeConversion> {
    path_arguments_to_types(arguments)
        .into_iter()
        .map(TypeConversion::from)
        .collect()
}

pub fn usize_to_tokenstream(value: usize) -> TokenStream2 {
    let lit = syn::LitInt::new(&value.to_string(), Span::call_site());
    lit.to_token_stream()
}

pub fn ident_from_item(item: &Item) -> Option<Ident> {
    match item {
        Item::Mod(item_mod) => Some(item_mod.ident.clone()),
        Item::Struct(item_struct) => Some(item_struct.ident.clone()),
        Item::Enum(item_enum) => Some(item_enum.ident.clone()),
        Item::Type(item_type) => Some(item_type.ident.clone()),
        Item::Fn(item_fn) => Some(item_fn.sig.ident.clone()),
        Item::Trait(item_trait) => Some(item_trait.ident.clone()),
        Item::Impl(item_impl) => type_ident(&item_impl.self_ty),
        Item::Use(item_use) => ItemConversion::fold_use(&item_use.tree).first().cloned().cloned(),
        _ => None,
    }
}

fn cache_fields_in(container: &mut HashMap<ImportConversion, HashSet<ImportComposition>>, fields: &Fields, imports: &HashMap<PathHolder, Path>) {
    match fields {
        Fields::Unnamed(FieldsUnnamed { unnamed: fields, .. }) |
        Fields::Named(FieldsNamed { named: fields, .. }) =>
            fields.iter()
                .for_each(|field| cache_type_in(container, &field.ty, imports)),
        Fields::Unit => {}
    }
}

fn cache_type_in(container: &mut HashMap<ImportConversion, HashSet<ImportComposition>>, ty: &Type, imports: &HashMap<PathHolder, Path>) {
    // Types which are used as a part of types (for generics and composite types)
    // let type_conversion = TypeHolder::from(ty);
    // let involved = <TypePathHolder as Conversion>::nested_items(ty, &VisitorContext::Unknown);
    // let involved = <TypeHolder as Conversion>::nested_items(ty);
    let involved: HashSet<Type> = ty.nested_items();
    involved.iter()
        .for_each(|ty| {
            let path: Path = parse_quote!(#ty);
            if let Some(PathSegment { ident, .. }) = path.segments.last() {
                let (import_type, scope) = import_pair(&path, imports);
                container
                    .entry(import_type)
                    .or_default()
                    .insert(ImportComposition::from((ident, &scope)));
            }
        });
}
fn import_pair(path: &Path, imports: &HashMap<PathHolder, Path>) -> (ImportConversion, PathHolder) {
    let original_or_external_pair = |value| {
        println!("import_pair:::value: {}", format_token_stream(value));
        let scope = PathHolder::from(value);
        (if scope.has_belong_to_current_crate() { ImportConversion::Original } else { ImportConversion::External }, scope)
    };
    let path_scope= PathHolder::from(path);
    println!("import_pair: {}", format_token_stream(path));
    match path.get_ident() {
        Some(ident) => match ident.to_string().as_str() {
            // accessible without specifying scope
            "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" | "f64" | "i128" | "u128" | "isize"
            | "usize" | "bool" | "str" | "String" | "Vec" | "Option" | "Box" =>
                (ImportConversion::None, parse_quote!(#ident)),
            // they are defined in the same scope, so it should be imported sometimes outside this scope (export-only)
            _ =>
                imports.get(&path_scope)
                    .map_or((ImportConversion::Inner, parse_quote!(#ident)), original_or_external_pair)
        },
        // partial chunk
        None => {
            imports.get(&path_scope)
                .map_or(match path.segments.last().unwrap().ident.to_string().as_str() {
                    "Vec" | "Option" | "Box" => (ImportConversion::None, path_scope),
                    _ => (ImportConversion::ExternalChunk, path_scope),
                }, original_or_external_pair)
        }
    }
}
pub fn is_labeled_with_macro_type(path: &Path, macro_type: &str) -> bool {
    path.segments
        .iter()
        .any(|segment| macro_type == segment.ident.to_string().as_str())
}
pub fn is_labeled_for_export(path: &Path) -> bool {
    is_labeled_with_macro_type(path, "export")
}
pub fn is_owner_labeled_with_trait_implementation(path: &Path) -> bool {
    is_labeled_with_macro_type(path, "export")
}

pub fn handle_attributes_with_handler<F: FnMut(MacroAttributes)>(attrs: &[Attribute], mut handler: F) {
    attrs.iter()
        .for_each(|attr|
            if is_labeled_for_export(&attr.path) || is_owner_labeled_with_trait_implementation(&attr.path) {
                let mut arguments = Vec::<Path>::new();
                if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                    meta_list.nested.iter().for_each(|meta| {
                        if let NestedMeta::Meta(Meta::Path(path)) = meta {
                            arguments.push(path.clone());
                        }
                    });
                }
                handler(MacroAttributes {
                    path: attr.path.clone(),
                    arguments
                })
            }
        )
}
pub fn collect_generic_types_in_path(path: &Path, generics: &mut HashSet<TypeHolder>) {
    path.segments
        .iter()
        .flat_map(|seg| path_arguments_to_types(&seg.arguments))
        .for_each(|t| collect_generic_types_in_type(&t, generics));
}

pub fn collect_bounds(bounds: &Punctuated<TypeParamBound, Add>) -> Vec<Path> {
    bounds.iter().filter_map(|bound| match bound {
        TypeParamBound::Trait(TraitBound { path, .. }) => Some(path.clone()),
        TypeParamBound::Lifetime(_lifetime) => None
    }).collect()
}



pub fn collect_generic_types_in_type(field_type: &Type, generics: &mut HashSet<TypeHolder>) {
    //println!("collect_generic_types_in_type.1: {}", field_type.to_token_stream());
    match field_type {
        Type::Path(TypePath { path, .. }) => {
            collect_generic_types_in_path(path, generics);
            if path.segments.iter().any(|seg| !path_arguments_to_types(&seg.arguments).is_empty() && !matches!(seg.ident.to_string().as_str(), "Option")) {
                // println!("addd generic: {}", format_token_stream(field_type));
                generics.insert(TypeHolder(field_type.clone()));
            }
        },
        Type::Reference(TypeReference { elem, .. }) => {
            collect_generic_types_in_type(elem, generics);
        },
        Type::TraitObject(TypeTraitObject { bounds, .. }) => {
            bounds.iter().for_each(|bound| match bound {
                TypeParamBound::Trait(TraitBound { path, .. }) => collect_generic_types_in_path(path, generics),
                _ => {}
            })
        },
        Type::Tuple(TypeTuple { elems, .. }) => {
            //println!("collect_generic_types_in_type.t: {}", field_type.to_token_stream());
            generics.insert(TypeHolder(field_type.clone()));
            elems.iter()
                .for_each(|ty| collect_generic_types_in_type(ty, generics));
        },
        // Type::Array ??
        _ => {}
    }
}

impl ItemExtension for Signature {
    fn scope_tree_export_id(&self) -> ScopeTreeExportID {
        ScopeTreeExportID::Ident(self.ident.clone())
    }

    fn maybe_attrs(&self) -> Option<&Vec<Attribute>> {
        None
    }

    fn maybe_ident(&self) -> Option<&Ident> {
       Some(&self.ident)
    }

    fn maybe_generics(&self) -> Option<&Generics> {
        Some(&self.generics)
    }

    fn collect_compositions(&self) -> Vec<TypeHolder> {
        let mut type_and_paths: Vec<TypeHolder> = Vec::new();
        self.inputs.iter().for_each(|arg|
            if let FnArg::Typed(PatType { ty, .. }) = arg {
                type_and_paths.push(TypeHolder(*ty.clone()));
            });
        if let ReturnType::Type(_, ty) = &self.output {
            type_and_paths.push(TypeHolder(*ty.clone()));
        }
        type_and_paths
    }

    fn classify_imports(&self, imports: &HashMap<PathHolder, Path>) -> HashMap<ImportConversion, HashSet<ImportComposition>> {
        let mut container = HashMap::new();
        self.inputs.iter().for_each(|arg| {
            if let FnArg::Typed(PatType { ty, .. }) = arg {
                cache_type_in(&mut container, ty, imports)
            }
        });
        if let ReturnType::Type(_, ty) = &self.output {
            cache_type_in(&mut container, ty, imports)
        };
        container
    }
}

