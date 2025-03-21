use std::rc::Rc;
use quote::{quote, ToTokens};
use syn::{Attribute, Generics, Lifetime, PathSegment, Type};
use ferment_macro::ComposerBase;
use crate::ast::{CommaPunctuated, Depunctuated, SemiPunctuated};
use crate::composable::{AttrsModel, FieldComposer, FieldTypeKind, GenModel, LifetimesModel};
use crate::composer::{AspectPresentable, AttrComposable, BasicComposer, BasicComposerLink, BasicComposerOwner, ComposerLink, GenericComposerInfo, SourceComposable, ToConversionFullComposer, VarComposer};
use crate::context::{ScopeContext, ScopeContextLink};
use crate::conversion::{DictFermentableModelKind, DictTypeModelKind, GenericTypeKind, SmartPointerModelKind, TypeKind, TypeModelKind};
use crate::ext::{CrateExtension, GenericNestedArg, LifetimeProcessor, Mangle, MaybeLambdaArgs, ToPath, ToType};
use crate::lang::{FromDictionary, LangFermentable, RustSpecification, Specification};
use crate::presentable::{Aspect, Expression, ScopeContextPresentable, TypeContext};
use crate::presentation::{DictionaryExpr, DictionaryName, DocComposer, InterfacePresentation, RustFermentate};

#[derive(ComposerBase)]
pub struct AnyOtherComposer<LANG, SPEC>
    where LANG: LangFermentable + 'static,
          SPEC: Specification<LANG> + 'static {
    pub ty: Type,
    base: BasicComposerLink<LANG, SPEC, Self>,
}

impl<LANG, SPEC> AnyOtherComposer<LANG, SPEC>
    where LANG: LangFermentable,
          SPEC: Specification<LANG> {
    pub fn new(ty: &Type, ty_context: SPEC::TYC, attrs: Vec<Attribute>, scope_context: &ScopeContextLink) -> Self {
        Self {
            base: BasicComposer::from(DocComposer::new(ty_context.to_token_stream()), AttrsModel::from(&attrs), ty_context, GenModel::default(), LifetimesModel::default(), Rc::clone(scope_context)),
            ty: ty.clone(),
        }
    }
}

impl<SPEC> SourceComposable for AnyOtherComposer<RustFermentate, SPEC>
    where SPEC: RustSpecification {
    type Source = ScopeContext;
    type Output = Option<GenericComposerInfo<RustFermentate, SPEC>>;

    fn compose(&self, source: &Self::Source) -> Self::Output {
        let mut lifetimes = Vec::<Lifetime>::new();
        let ffi_name = self.ty.mangle_tokens_default();
        let arg_0_name = SPEC::Name::dictionary_name(DictionaryName::Obj);

        let path = self.ty.to_path();
        let ctor_path = path.arg_less();

        // Arc/Rc: primitive arg: to: "*obj"
        // Arc/Rc: complex arg: to: "(*obj).clone()"
        // Mutex/RwLock: primitive/complex arg: to: "obj.into_inner().expect("Err")"
        // Arc<RwLock>>: to: obj.borrow().clone()
        // RefCell: primitive/complex arg: to: "obj.into_inner()"
        let obj_by_value = source.maybe_object_by_value(&self.ty);
        let nested_ty = self.ty.maybe_first_nested_type_ref()?;
        lifetimes.extend(nested_ty.unique_lifetimes());
        let maybe_opaque = source.maybe_opaque_object::<RustFermentate, SPEC>(nested_ty);
        let nested_obj_by_value = source.maybe_object_by_value(nested_ty);
        println!("AnyOther.ty: {}", nested_ty.to_token_stream());
        println!("AnyOther.nested.ty: {}", nested_ty.to_token_stream());
        println!("AnyOther by_value: {}", obj_by_value.as_ref().map_or("None".to_string(), |o| format!("{o}")));
        println!("AnyOther nested: by_value: {}", nested_obj_by_value.as_ref().map_or("None".to_string(), |o| format!("{o}")));
        println!("AnyOther opaque: {}", maybe_opaque.to_token_stream());
        // let _maybe_nested_nested_ty = nested_ty.maybe_first_nested_type_ref();
        // let compose = |arg_name: &Name, ty: &Type| {
        // };
        // let arg_name = &arg_0_name;
        // let ty = nested_ty;
        // compose(&arg_0_name, nested_ty)

        let ffi_var = VarComposer::<RustFermentate, SPEC>::value(nested_ty)
            .compose(source)
            .to_type();
        let maybe_obj = source.maybe_object_by_value(nested_ty);
        let maybe_opaque = source.maybe_opaque_object::<RustFermentate, SPEC>(nested_ty);
        let is_opaque = maybe_opaque.is_some();
        // println!("compose ffi_type: {}", ffi_var.to_token_stream());
        let to_expr = {
            match &path.segments.last() {
                Some(PathSegment { ident, .. }) => match ident.to_string().as_str() {
                    "Arc" | "Rc" => {
                        match TypeKind::from(nested_ty) {
                            TypeKind::Primitive(_) =>
                                DictionaryExpr::Deref(arg_0_name.to_token_stream()).to_token_stream(),
                            TypeKind::Complex(_) => {
                                if maybe_opaque.is_some() {
                                    quote!(#ctor_path::into_raw(#arg_0_name).cast_mut())
                                } else {
                                    quote!((*#arg_0_name).clone())
                                }
                            },
                            TypeKind::Generic(nested_generic_ty) => {
                                // println!("GENERIC inside Arc/Rc: {}", nested_generic_ty);
                                match nested_generic_ty {
                                    GenericTypeKind::AnyOther(ty) => {
                                        // println!("GENERIC (AnyOther) inside Arc/Rc: {}", ty.to_token_stream());
                                        let path = ty.to_path();
                                        match &path.segments.last() {
                                            Some(PathSegment { ident, .. }) => match ident.to_string().as_str() {
                                                "RwLock" | "Mutex" => quote!(std::sync::#ident::new(obj.read().expect("Poisoned").clone())),
                                                _ => quote!((*#arg_0_name).clone())
                                            },
                                            None => quote!((*#arg_0_name).clone())
                                        }
                                    },
                                    _ => quote!((*#arg_0_name).clone())
                                }
                            },
                        }
                    },
                    "Mutex" | "RwLock" => {

                        let expr = ToConversionFullComposer::<RustFermentate, SPEC>::value(arg_0_name.clone(), nested_ty).compose(source);
                        println!("RES expr: {}", expr.present(source));
                        // let pres = expr.present(source);
                        // Expression::
                        quote!(#arg_0_name.into_inner().expect("Err"))
                    },
                    "RefCell" => quote!(#arg_0_name.into_inner()),
                    "Pin" => quote!(&**#arg_0_name),
                    _ => quote!((*#arg_0_name).clone())
                }
                None => quote!((*#arg_0_name).clone())
            }
        };
        let (from_conversion, to_conversion, destroy_conversion) = match maybe_obj.as_ref().and_then(|o| o.maybe_type_model_kind_ref()) {
            Some(ty_model_kind) => match ty_model_kind {
                TypeModelKind::Dictionary(DictTypeModelKind::Primitive(..)) => (
                    Expression::<RustFermentate, SPEC>::from_primitive_tokens(quote!(ffi_ref.#arg_0_name)),
                    Some(Expression::<RustFermentate, SPEC>::ffi_to_primitive_tokens(to_expr)),
                    Expression::<RustFermentate, SPEC>::destroy_primitive_tokens(DictionaryExpr::SelfProp(arg_0_name.to_token_stream()))
                ),
                TypeModelKind::Dictionary(DictTypeModelKind::LambdaFn(..)) => {
                    if let Some(lambda_args) = MaybeLambdaArgs::<RustFermentate, SPEC>::maybe_lambda_arg_names(ty_model_kind) {
                        (Expression::from_lambda(Expression::Simple(quote!((&*ffi_ref.#arg_0_name))), lambda_args), None, Expression::destroy_primitive_tokens(DictionaryExpr::SelfProp(arg_0_name.to_token_stream())))
                    } else {
                        (Expression::from_primitive(Expression::<RustFermentate, SPEC>::FfiRefWithName(arg_0_name.clone()).into()), Some(Expression::ffi_to_primitive_tokens(to_expr)), Expression::destroy_primitive_tokens(DictionaryExpr::SelfProp(arg_0_name.to_token_stream())))
                    }
                },
                TypeModelKind::Dictionary(DictTypeModelKind::NonPrimitiveOpaque(..)) =>
                    (Expression::from_primitive_tokens(quote!(ffi_ref.#arg_0_name)), Some(Expression::ffi_to_primitive_tokens(to_expr)), Expression::destroy_complex_tokens(DictionaryExpr::SelfProp(arg_0_name.to_token_stream()))),
                TypeModelKind::Dictionary(DictTypeModelKind::NonPrimitiveFermentable(kind)) => match kind {
                    DictFermentableModelKind::SmartPointer(SmartPointerModelKind::Box(..)) => (Expression::from_complex_opt(Expression::FfiRefWithName(arg_0_name.clone()).into()), Some(Expression::ffi_to_complex_opt_tokens(to_expr)), Expression::destroy_complex_opt_tokens(DictionaryExpr::SelfProp(arg_0_name.to_token_stream()))),
                    _ if is_opaque => (
                        Expression::from_primitive_tokens(quote!(ffi_ref.#arg_0_name)),
                        Some(Expression::ffi_to_primitive_tokens(to_expr)),
                        Expression::destroy_complex_tokens(DictionaryExpr::SelfProp(arg_0_name.to_token_stream()))
                    ),
                    _ => (
                        Expression::from_complex_tokens(quote!(ffi_ref.#arg_0_name)),
                        Some(Expression::ffi_to_complex_tokens(to_expr)),
                        Expression::destroy_complex_tokens(DictionaryExpr::SelfProp(arg_0_name.to_token_stream()))
                    )
                },
                TypeModelKind::Optional(model) => match model.first_nested_argument() {
                    Some(nested_arg) => match nested_arg.maybe_type_model_kind_ref() {
                        Some(nested_ty_model_kind) => match nested_ty_model_kind {
                            TypeModelKind::Dictionary(DictTypeModelKind::Primitive(..)) =>
                                (Expression::from_primitive_opt_tokens(quote!(ffi_ref.#arg_0_name)), Some(Expression::ffi_to_primitive_opt_tokens(to_expr)), Expression::destroy_primitive_opt_tokens(DictionaryExpr::SelfProp(arg_0_name.to_token_stream()))),
                            TypeModelKind::Dictionary(DictTypeModelKind::NonPrimitiveFermentable(DictFermentableModelKind::SmartPointer(SmartPointerModelKind::Box(..)))) => {
                                (Expression::map_into_box(Expression::from_complex_opt_tokens(quote!(ffi_ref.#arg_0_name))), Some(Expression::ffi_to_complex_opt_tokens(to_expr)), Expression::destroy_complex_opt_tokens(DictionaryExpr::SelfProp(arg_0_name.to_token_stream())))
                            },
                            _ => (Expression::from_complex_opt_tokens(quote!(ffi_ref.#arg_0_name)), Some(Expression::ffi_to_complex_opt_tokens(to_expr)), Expression::destroy_complex_opt_tokens(DictionaryExpr::SelfProp(arg_0_name.to_token_stream()))),
                        },
                        _ => (Expression::from_primitive_tokens(quote!(ffi_ref.#arg_0_name)), Some(Expression::ffi_to_primitive_tokens(to_expr)), Expression::destroy_primitive_tokens(DictionaryExpr::SelfProp(arg_0_name.to_token_stream()))),
                    },
                    _ => (Expression::from_complex_opt_tokens(quote!(ffi_ref.#arg_0_name)), Some(Expression::ffi_to_complex_opt_tokens(to_expr)), Expression::destroy_complex_opt_tokens(DictionaryExpr::SelfProp(arg_0_name.to_token_stream()))),
                },
                _ if is_opaque => (Expression::from_primitive_tokens(quote!(ffi_ref.#arg_0_name)), Some(Expression::ffi_to_primitive_tokens(to_expr)), Expression::destroy_complex_tokens(DictionaryExpr::SelfProp(arg_0_name.to_token_stream()))),
                _ => (Expression::from_complex_tokens(quote!(ffi_ref.#arg_0_name)), Some(Expression::ffi_to_complex_tokens(to_expr)), Expression::destroy_complex_tokens(DictionaryExpr::SelfProp(arg_0_name.to_token_stream()))),
            },
            None => (Expression::from_primitive_tokens(quote!(ffi_ref.#arg_0_name)), Some(Expression::ffi_to_primitive_tokens(to_expr)), Expression::destroy_primitive_tokens(DictionaryExpr::SelfProp(arg_0_name.to_token_stream())))
        };

        let types = (self.present_ffi_aspect(), self.present_target_aspect());
        let attrs = self.compose_attributes();
        let mut interfaces = Depunctuated::new();
        let from_body = {
            let conversion = from_conversion.present(source);
            let from = maybe_opaque.as_ref().map_or(quote!(new), |_| quote!(from_raw));
            quote!(#ctor_path::#from(#conversion))
        };
        interfaces.push(InterfacePresentation::conversion_from_root(&attrs, &types, from_body, &None, &lifetimes));
        if let Some(to_conversion) = to_conversion {
            let expr_to_iter = [
                FieldComposer::<RustFermentate, SPEC>::named(arg_0_name.clone(), FieldTypeKind::Conversion(Expression::<RustFermentate, SPEC>::present(&to_conversion, source)))
            ];
            let to_body = CommaPunctuated::from_iter(expr_to_iter).present(source);
            interfaces.push(InterfacePresentation::conversion_to_boxed_self_destructured(&attrs, &types, to_body, &None, &lifetimes));
        }
        let field_composers = Depunctuated::from_iter([
            FieldComposer::<RustFermentate, SPEC>::named(arg_0_name.clone(), FieldTypeKind::Type(ffi_var))
        ]);
        let expr_destroy_iterator = [
            destroy_conversion.present(source)
        ];
        interfaces.push(InterfacePresentation::drop(&attrs, ffi_name.to_type(), SemiPunctuated::from_iter(expr_destroy_iterator)));
        let aspect = Aspect::RawTarget(TypeContext::Struct { ident: self.ty.mangle_ident_default(), attrs: vec![], generics: Generics::default() });
        Some(GenericComposerInfo::<RustFermentate, SPEC>::default(aspect, &attrs, field_composers, interfaces))
    }
}

