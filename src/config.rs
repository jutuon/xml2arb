use std::{path::PathBuf, str::FromStr};

use clap::{arg, Parser};

pub const ARB_FILE_TEMPLATE_SUFFIX: &str = "_en.arb";

#[derive(Parser)]
#[command(author, version, about)]
pub struct ArgsConfig {
    /// Path to Android app project "res" directory.
    /// The directory must contain "values/strings.xml" and optional
    /// "values-language/strings.xml" files where "language" is for example
    /// a two-letter country code.
    #[arg(long, value_name = "DIR")]
    pub input_dir: PathBuf,

    /// Directory for generated ARB files. The directory is created
    /// if it does not exist.
    #[arg(long, value_name = "DIR")]
    pub output_dir: PathBuf,

    /// Template file name for generated ARB files.
    /// This must end with "_en.arb". The "en" part is replaced with the
    /// language for each strings.xml file. The "values/strings.xml" has
    /// the language "en" and "values-language/strings.xml" has
    /// "language" as the language.
    #[arg(long, value_parser = ArbFileNameTemplate::from_str)]
    pub arb_file_name_template: ArbFileNameTemplate,
}

pub fn get_config() -> ArgsConfig {
    ArgsConfig::parse()
}

#[derive(Debug, Clone)]
pub struct ArbFileNameTemplate {
    /// File name before "_en.arb"
    prefix: String,
}

impl ArbFileNameTemplate {
    pub fn new(template: &str) -> Result<Self, String> {
        if !template.ends_with(ARB_FILE_TEMPLATE_SUFFIX) {
            return Err(format!(
                "Template file name must end with '{}'",
                ARB_FILE_TEMPLATE_SUFFIX
            ));
        }
        let prefix = template
            .trim_end_matches(ARB_FILE_TEMPLATE_SUFFIX)
            .to_string();
        Ok(ArbFileNameTemplate { prefix })
    }

    pub fn get_file_name(&self, locale: &str) -> String {
        format!("{}_{}.arb", self.prefix, locale)
    }
}

impl FromStr for ArbFileNameTemplate {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        ArbFileNameTemplate::new(value)
    }
}
