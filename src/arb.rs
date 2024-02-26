use std::path::Path;

use anyhow::Result;
use serde_json::json;

use crate::{config::ArbFileNameTemplate, xml::{ParsedStringsXml, StringValue, LOCALE_EN}};

pub fn save_to_arb_files(arb_dir: impl AsRef<Path>, strings: Vec<ParsedStringsXml>, arb_template: ArbFileNameTemplate) -> Result<()> {
    let path = arb_dir.as_ref();
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }

    for parsed in strings {
        let arb_file_content = to_arb_string(parsed.strings, parsed.locale != LOCALE_EN)?;
        let file_name = arb_template.get_file_name(&parsed.locale);
        let file_path = path.join(file_name);
        std::fs::write(file_path, arb_file_content)?;
    }

    Ok(())
}

fn to_arb_string(mut parsed: Vec<StringValue>, only_key: bool) -> Result<String> {
    parsed.sort_by(|a, b| a.key.cmp(&b.key));

    let mut map = serde_json::Map::new();

    for string in parsed {
        map.insert(string.key.clone(), json!(string.value));
        if only_key {
            continue;
        }
        if let Some(description) = string.description {
            map.insert(format!("@{}", string.key), json!({"description": description}));
        }
    }

    let mut arb_string = serde_json::to_string_pretty(&map)?;
    arb_string.push('\n');
    Ok(arb_string)
}
