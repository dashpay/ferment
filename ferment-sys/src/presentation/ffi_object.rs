use quote::{quote, ToTokens};
use proc_macro2::{TokenStream as TokenStream2};
use syn::{Attribute, Path};
use crate::presentation::present_struct;

#[derive(Clone, Debug)]
pub enum FFIObjectPresentation {
    // Empty,
    // StaticVTable {
    //     name: Name,
    //     methods_declarations: Punctuated<TokenStream2, Comma>,
    //     methods_implementations: Depunctuated<BindingPresentation>,
    //     // methods_compositions: Vec<TraitVTableMethodComposition>,
    //     // methods_names: Vec<Ident>,
    //     // methods_signatures: Vec<TokenStream2>,
    //     fq_trait_vtable: TokenStream2,
    //     // methods_implementations: Vec<TraitVTablePresentation>,
    //     // methods_declarations: Vec<TraitVTablePresentation>,
    // },
    TraitVTable {
        name: Path,
        attrs: Vec<Attribute>,
        fields: TokenStream2
    },
    // TraitVTableInnerFn {
    //     name: Name,
    //     name_and_args: TokenStream2,
    //     output_expression: ReturnType,
    // },
    TraitObject {
        name: Path,
        attrs: Vec<Attribute>,
        fields: TokenStream2
    },
    Full(TokenStream2),
    Empty,
    // Generic {
    //     object_presentation: TokenStream2,
    //     interface_presentations: Depunctuated<InterfacePresentation>,
    //     drop_presentation: DropInterfacePresentation,
    //     bindings: Depunctuated<BindingPresentation>,
    // },
}


impl ToTokens for FFIObjectPresentation {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Empty => quote!(),
            Self::Full(presentation) => quote!(#presentation),
            Self::TraitVTable { name, attrs, fields } => {
                present_struct(&name.segments.last().unwrap().ident, attrs, fields)
            },
            Self::TraitObject { name, attrs, fields } => {
                present_struct(&name.segments.last().unwrap().ident, attrs, fields)
            },
            // Self::Generic {
            //     object_presentation,
            //     interface_presentations,
            //     drop_presentation,
            //     bindings
            // } => quote! {
            //     #object_presentation
            //     #interface_presentations
            //     #drop_presentation
            //     #bindings
            // },
            // Self::Empty => { /* Box<T> */
            //     quote!()
            // },
/*            Self::StaticVTable { name, fq_trait_vtable, methods_declarations, methods_implementations } => {
                println!("FFIObjectPresentation::StaticVTable:: {:?} [{}]", name, fq_trait_vtable);
                quote! {
                    static #name: #fq_trait_vtable = {
                        #methods_implementations
                        #fq_trait_vtable {
                            #methods_declarations
                        }
                    };
                }
            }
*/        }.to_tokens(tokens)
    }
}
// # [doc = r" # Safety"]
// # [no_mangle]
// pub unsafe extern "C" fn Status_as_CanRetry_can_retry (obj: * const Status) -> bool {
//     let obj = ferment::FFIConversionFrom::ffi_from_const(obj);
//     let result = <crate::transport::transport_request::Status as crate::transport::transport_request::CanRetry>::can_retry(&obj);
//     result
// }
