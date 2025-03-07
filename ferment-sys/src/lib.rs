mod ast;
mod builder;
mod composable;
mod composer;
mod context;
mod conversion;
mod ext;
mod error;
mod formatter;
mod presentable;
mod presentation;
mod shared;
#[cfg(test)]
mod test;
mod tree;
mod lang;
mod config;

pub use self::error::Error;
pub use self::builder::Builder;
pub use self::config::Config;
pub use self::lang::{Lang, rust::Crate};
pub use self::builder::Builder as Ferment;
// pub use self::tree::create_headers;

#[cfg(feature = "objc")]
pub use self::lang::objc::Config as ObjC;
#[cfg(feature = "objc")]
pub use self::lang::objc::XCodeConfig;
#[cfg(feature = "java")]
pub use self::lang::java::Config as Java;

// It's organized as a sequential process of tree transformation
// Files -> File Tree -> Scope Agnostic Tree -> Full Context Tree -> Fermentate