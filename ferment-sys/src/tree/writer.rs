use std::fs::File;
use std::io::Write;
use std::process::Command;
use quote::ToTokens;
use crate::{Config, Crate, Error, Lang};
#[cfg(not(feature = "cbindgen_only"))]
use crate::ast::Depunctuated;
#[cfg(not(feature = "cbindgen_only"))]
use crate::presentation::Fermentate;
use crate::presentation::RustFermentate;

pub trait IWriter {
    type Fermentate: ToTokens;
    fn write(&self, fermentate: Self::Fermentate) -> Result<(), Error>;
}

pub struct Writer {
    config: Config
}

impl From<Config> for Writer {
    fn from(config: Config) -> Self {
        Self { config }
    }
}
impl From<&Config> for Writer {
    fn from(config: &Config) -> Self {
        Self { config: config.clone() }
    }
}

impl IWriter for Writer {
    type Fermentate = RustFermentate;

    fn write(&self, fermentate: Self::Fermentate) -> Result<(), Error> {
        File::create(self.config.expansion_path())
            .map_err(Error::from)
            .and_then(|mut output| output.write_all(fermentate.to_token_stream().to_string().as_bytes())
                .map_err(Error::from))
    }
}

impl Writer {

    pub(crate) fn write_headers(&self) -> Result<(), Error> {
        let Config { current_crate: Crate { name: framework, .. }, languages, cbindgen_config_from_file, .. } = &self.config;
        #[allow(irrefutable_let_patterns)]
        #[cfg(feature = "objc")]
        let framework = languages.iter().find_map(|l| if let Lang::ObjC(conf) = l { Some(&conf.xcode.header_name) } else { None }).unwrap_or(framework);
        Command::new("mkdir")
            .args(&["-p", "target/include"])
            .status()?;
        Command::new("cbindgen")
            .args([
                "--config", cbindgen_config_from_file.as_ref().map(String::as_str).unwrap_or("cbindgen.toml"),
                "-o", format!("target/include/{framework}.h").as_str()
            ])
            .status()
            .map_err(Error::from)
            .map(|_| ())
    }
    #[cfg(not(feature = "cbindgen_only"))]
    pub fn write(&self, fermentate: Depunctuated<Fermentate>) -> Result<(), Error> {
        for f in fermentate {
            match f {
                Fermentate::Rust(fermentate) =>
                    IWriter::write(self, fermentate)?,
                #[cfg(feature = "objc")]
                Fermentate::ObjC(fermentate) => if let Some(config) = self.config.maybe_objc_config() {
                    crate::lang::objc::ObjCWriter::from(config)
                        .write(fermentate)?

                }
                _ => {}
            }
        }
        Ok(())
    }
}

