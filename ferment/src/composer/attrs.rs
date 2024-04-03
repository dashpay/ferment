use quote::{format_ident, quote, ToTokens};
use syn::{ItemTrait, parse_quote};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Paren};
use crate::composer::{Composer, Depunctuated};
use crate::composition::{AttrsComposition, FnReturnTypeComposition, TraitDecompositionPart2, TraitVTableMethodComposition, TypeComposition};
use crate::conversion::TypeCompositionConversion;
use crate::context::{ScopeChain, ScopeContext};
use crate::holder::EMPTY;
use crate::naming::Name;
use crate::presentation::context::OwnedItemPresentableContext;
use crate::presentation::{BindingPresentation, FFIObjectPresentation, ScopeContextPresentable, TraitVTablePresentation};
use crate::shared::{HasParent, SharedAccess};
use crate::wrapped::Wrapped;

// #[derive(Parent<'a>)]
pub struct AttrsComposer<Parent: SharedAccess> {
    pub parent: Option<Parent>,
    pub attrs: AttrsComposition,
}

impl<Parent: SharedAccess> AttrsComposer<Parent> {
    pub fn new(attrs: AttrsComposition) -> AttrsComposer<Parent> {
        Self { parent: None, attrs }
    }
}

impl<Parent> HasParent<Parent> for AttrsComposer<Parent> where Parent: SharedAccess {
    fn set_parent(&mut self, parent: &Parent) {
        self.parent = Some(parent.clone_container());
    }
}

impl<Parent: SharedAccess> Composer<Parent> for AttrsComposer<Parent> {
    type Source = ScopeContext;
    type Result = Depunctuated<TraitVTablePresentation>;

    fn compose(&self, source: &Self::Source) -> Self::Result {
        let attrs_composition = &self.attrs;
        let mut trait_types = source.trait_items_from_attributes(&attrs_composition.attrs);
        trait_types.iter_mut()
            .map(|(composition, trait_scope)| {
                // TODO: move to full and replace nested_arguments
                let conversion = TypeCompositionConversion::Object(TypeComposition::new(source.scope.to_type(), Some(composition.item.generics.clone()), Punctuated::new()));
                println!("AttrsComposer: {} {} {}", composition.item.ident, trait_scope, conversion);
                composition.implementors.push(conversion);
                implement_trait_for_item((&composition.item, trait_scope), attrs_composition, source)
            })
            .collect()
    }
}

pub fn implement_trait_for_item(item_trait: (&ItemTrait, &ScopeChain), attrs_composition: &AttrsComposition, context: &ScopeContext) -> TraitVTablePresentation {
    let (item_trait, trait_scope) = item_trait;
    let AttrsComposition { ident: item_name, scope: item_scope, .. } = attrs_composition;
    let trait_decomposition = TraitDecompositionPart2::from_trait_items(&item_trait.items, Some(parse_quote!(#item_name)), &EMPTY, context);
    let trait_ident = &item_trait.ident;
    let item_full_ty = context.full_type_for(&parse_quote!(#item_name));
    let trait_full_ty = context.full_type_for(&parse_quote!(#trait_ident));
    let methods_compositions: Vec<TraitVTableMethodComposition> = trait_decomposition.methods.into_iter()
        .map(|signature_decomposition| {
            let FnReturnTypeComposition { presentation: output_expression, conversion: output_conversions } = signature_decomposition.return_type;
            let fn_name = signature_decomposition.ident.unwrap();
            let ffi_fn_name = format_ident!("{}_{}", item_name, fn_name);
            let arguments = signature_decomposition.arguments
                .iter()
                .map(|arg| arg.name_type_original.clone())
                .collect::<Punctuated<_, Comma>>();

            let arguments_presentation = Wrapped::<_, Paren>::new(arguments.present(context));
            // OwnerIteratorPresentationContext::RoundBracesFields(local_context)
            // let arguments_presentation = constants::ROUND_BRACES_FIELDS_PRESENTER((Name::Just(quote!(unsafe extern "C" fn #ffi_fn_name)), arguments)).present(context);
            let argument_conversions = signature_decomposition
                .arguments
                .iter()
                .map(|arg| OwnedItemPresentableContext::Conversion(if arg.name.is_some() {
                    quote!(cast_obj)
                } else {
                    arg.name_type_conversion.present(context)
                }))
                .collect::<Punctuated<_, Comma>>();
            let argument_names = Wrapped::<_, Paren>::new(argument_conversions.present(context))
                .to_token_stream();


            TraitVTableMethodComposition {
                name_and_args: quote!(unsafe extern "C" fn #ffi_fn_name #arguments_presentation),
                fn_name,
                ffi_fn_name,
                item_type: item_full_ty.clone(),
                trait_type: trait_full_ty.clone(),
                output_expression,
                output_conversions: output_conversions.present(context),
                argument_names
            }
        }).collect();
    let trait_vtable_ident = Name::Vtable(trait_ident.clone());
    let trait_object_ident = Name::TraitObj(trait_ident.clone());
    let other = trait_scope;
    let trait_self_scope = other.self_path_holder();
    let is_defined_in_same_scope = item_scope.has_same_parent(&other);
    let full_trait_type = if is_defined_in_same_scope { quote!(#trait_object_ident) } else { quote!(#trait_self_scope::#trait_object_ident) };
    let vtable_name = Name::TraitImplVtable(item_name.clone(), trait_ident.clone());
    TraitVTablePresentation::Full {
        vtable: FFIObjectPresentation::StaticVTable {
            name: vtable_name.clone(),
            fq_trait_vtable: if is_defined_in_same_scope { quote!(#trait_vtable_ident) } else { quote!(#trait_self_scope::#trait_vtable_ident) },
            methods_compositions,
        },
        export: BindingPresentation::ObjAsTrait {
            name: Name::TraitFn(item_name.clone(), trait_ident.clone()),
            item_type: item_full_ty.to_token_stream(),
            trait_type: full_trait_type.to_token_stream(),
            vtable_name,
        },
        destructor: BindingPresentation::ObjAsTraitDestructor {
            name: Name::TraitDestructor(item_name.clone(), trait_ident.clone()),
            item_type: item_full_ty.to_token_stream(),
            trait_type: full_trait_type.to_token_stream(),
        }
    }
}
