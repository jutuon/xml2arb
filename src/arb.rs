use std::path::Path;

use anyhow::Result;
use serde_json::{json, Value};

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

        if file_path.exists() {
            let current_file_content = std::fs::read_to_string(&file_path)?;
            if current_file_content == arb_file_content {
                continue;
            }
        }

        std::fs::write(file_path, arb_file_content)?;
    }

    Ok(())
}

fn to_arb_string(mut parsed: Vec<StringValue>, only_key: bool) -> Result<String> {
    parsed.sort_by(|a, b| a.key.cmp(&b.key));

    let mut map = serde_json::Map::new();

    for string in parsed {
        map.insert(string.key.clone(), json!(string.value.to_arb_string()));
        if only_key {
            continue;
        }

        let mut attributes = serde_json::Map::new();

        if let Some(description) = string.description {
            attributes.insert("description".to_string(), Value::String(description));
        }

        if string.value.format_specifiers().any(|_| true) {
            let mut placeholdres = serde_json::Map::new();
            for specifier in string.value.format_specifiers() {
                let k = specifier.to_arb_placeholder_name();
                let placeholder_type = specifier.to_arb_placeholder_type();
                placeholdres.insert(
                    k,
                    json!({
                        "type" : placeholder_type
                    })
                );
            }
            attributes.insert("placeholders".to_string(), Value::Object(placeholdres));
        }


        if !attributes.is_empty() {
            map.insert(format!("@{}", string.key), Value::Object(attributes));
        }
    }

    let mut arb_string = serde_json::to_string_pretty(&map)?;
    arb_string.push('\n');
    Ok(arb_string)
}
