use syn::{AngleBracketedGenericArguments, GenericArgument, PathArguments, Type};
use crate::conversion::TypeKind;

#[allow(unused)]
pub trait GenericNestedArg {
    fn maybe_first_nested_type_ref(&self) -> Option<&Type>;
    fn nested_types(&self) -> Vec<&Type>;

    fn maybe_first_nested_type_kind(&self) -> Option<TypeKind> {
        self.maybe_first_nested_type_ref()
            .map(TypeKind::from)
    }
}

impl GenericNestedArg for Type {
    fn maybe_first_nested_type_ref(&self) -> Option<&Type> {
        match self {
            Type::Array(type_array) => Some(&type_array.elem),
            Type::Slice(type_slice) => Some(&type_slice.elem),
            Type::Reference(type_reference) => Some(&type_reference.elem),
            Type::Path(type_path) => type_path.path.segments.last().and_then(|last_segment| match &last_segment.arguments {
                PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =>
                    args.iter().find_map(|arg| match arg {
                        GenericArgument::Type(ty) => Some(ty),
                        _ => None
                    }),
                _ => None,
            }),
            // Type::BareFn(_) => {}
            // Type::Group(_) => {}
            // Type::ImplTrait(_) => {}
            // Type::Infer(_) => {}
            // Type::Macro(_) => {}
            // Type::Never(_) => {}
            // Type::Paren(_) => {},
            // Type::Ptr(_) => {}
            // Type::Reference(_) => {}
            // Type::Slice(_) => {}
            // Type::TraitObject(_) => {}
            // Type::Tuple(_) => {}
            // Type::Verbatim(_) => {}
            // Type::__NonExhaustive => {}
            _ => None
        }
    }

    fn nested_types(&self) -> Vec<&Type> {
        match self {
            Type::Array(type_array) => vec![&type_array.elem],
            Type::Slice(type_slice) => vec![&type_slice.elem],
            Type::Reference(type_reference) => vec![&type_reference.elem],
            Type::Path(type_path) => {
                let mut vec = Vec::<&Type>::new();
                if let Some(last_segment) = type_path.path.segments.last() {
                    match &last_segment.arguments {
                        PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =>
                            args.iter().for_each(|arg| match arg {
                                GenericArgument::Type(ty) => vec.push(ty),
                                _ => {}
                            }),
                        _ => {}
                    }
                }
                vec
            },
            Type::Tuple(type_tuple) => type_tuple.elems.iter().collect(),
            _ => vec![]
        }
    }
}

