use quote::ToTokens;
use syn::{parse_quote, Type};
use syn::__private::TokenStream2;

pub trait Accessory: ToTokens {
    fn joined_mut(&self) -> Self;
    fn joined_const(&self) -> Self;
}
#[macro_export]
macro_rules! impl_accessory {
    ($ty:ty) => {
        impl crate::ext::Accessory for $ty {
            fn joined_mut(&self) -> Self {
                parse_quote!(*mut #self)
            }

            fn joined_const(&self) -> Self {
                parse_quote!(*const #self)
            }
        }
    };
}
impl_accessory!(Type);
impl_accessory!(TokenStream2);
