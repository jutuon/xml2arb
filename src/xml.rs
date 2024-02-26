//! Read Android string resources from XML files

use std::path::Path;
use std::fs;
use anyhow::{anyhow, Context, Result};
use xml::attribute::OwnedAttribute;

pub const LOCALE_EN: &str = "en";
const VALUES_DIR_NAME: &str = "values";
const VALUES_DIR_NAME_WITH_LOCALE_PREFIX: &str = "values-";
const STRINGS_XML_FILE_NAME: &str = "strings.xml";
const STRINGS_XML_RESOURCES_TAG: &str = "resources";
const STRINGS_XML_STRING_TAG: &str = "string";
const STRINGS_XML_NAME_ATTRIBUTE: &str = "name";
const STRINGS_XML_DESCRIPTION_ATTRIBUTE: &str = "description";

#[derive(Debug)]
pub struct StringValue {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct ParsedStringsXml {
    pub strings: Vec<StringValue>,
    pub locale: String,
}

pub fn parse_android_strings_xml_files(android_project_res_dir: impl AsRef<Path>) -> Result<Vec<ParsedStringsXml>> {
    let mut parsed_files = Vec::new();

    for entry in fs::read_dir(android_project_res_dir)? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }

        let strings_xml_path = entry.path().join(STRINGS_XML_FILE_NAME);
        if !strings_xml_path.is_file() {
            continue;
        }

        let dir_file_name = entry.file_name();
        let locale = match dir_file_name.to_str() {
            Some(VALUES_DIR_NAME) => LOCALE_EN,
            Some(other) =>
                if other.starts_with(VALUES_DIR_NAME_WITH_LOCALE_PREFIX) {
                    other.trim_start_matches(VALUES_DIR_NAME_WITH_LOCALE_PREFIX)
                } else {
                    continue;
                }
            _ => continue,
        };

        let parsed_xml = handle_strings_xml(strings_xml_path, locale)?;
        parsed_files.push(parsed_xml);
    }

    Ok(parsed_files)
}

enum ParserState {
    Start,
    InResources,
    InString,
}

fn handle_strings_xml(xml: impl AsRef<Path>, locale: &str) -> Result<ParsedStringsXml> {
    let path = xml.as_ref();
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read strings.xml at {:?}", path))?;

    let mut state = ParserState::Start;
    let mut parsed_xml = ParsedStringsXml {
        strings: Vec::new(),
        locale: locale.to_string(),
    };
    let mut current_string = None;
    let parser = xml::EventReader::from_str(&contents);
    for e in parser {
        match e? {
            xml::reader::XmlEvent::StartElement { name, attributes, .. } =>
                match state {
                    ParserState::Start =>
                        if name.local_name == STRINGS_XML_RESOURCES_TAG {
                            state = ParserState::InResources;
                        }
                    ParserState::InResources =>
                        if name.local_name == STRINGS_XML_STRING_TAG {
                            state = ParserState::InString;
                            let mut string_value = StringValue {
                                key: String::new(),
                                value: String::new(),
                                description: None,
                            };
                            handle_string_tag_attributes(&mut string_value, &attributes)?;
                            current_string = Some(string_value);
                        }
                    ParserState::InString => (),
                }
            xml::reader::XmlEvent::EndElement { name } =>
                match state {
                    ParserState::Start |
                    ParserState::InResources => (),
                    ParserState::InString =>
                        if name.local_name == STRINGS_XML_STRING_TAG {
                            state = ParserState::InResources;
                        }
                }
            xml::reader::XmlEvent::Characters(s) =>
                match state {
                    ParserState::Start |
                    ParserState::InResources => (),
                    ParserState::InString =>
                        if let Some(mut current_string) = current_string.take() {
                            current_string.value = s;
                            parsed_xml.strings.push(current_string);
                        }
                }
            _ => {}
        }
    }
    Ok(parsed_xml)
}


fn handle_string_tag_attributes(value: &mut StringValue, tags: &[OwnedAttribute]) -> Result<()> {
    let name_tag = tags.iter()
        .find(|a| a.name.local_name == STRINGS_XML_NAME_ATTRIBUTE)
        .ok_or(anyhow!("Missing '{STRINGS_XML_NAME_ATTRIBUTE}' attribute"))?;
    value.key = name_tag.value.clone();

    let description = tags.iter().find(|a| a.name.local_name == STRINGS_XML_DESCRIPTION_ATTRIBUTE);
    if let Some(description) = description {
        value.description = Some(description.value.clone());
    }
    Ok(())
}
