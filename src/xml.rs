//! Read Android string resources from XML files

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::{path::Path, str::Chars};
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
    pub value: ParsedStringXmlValue,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct ParsedStringsXml {
    pub strings: Vec<StringValue>,
    pub locale: String,
}

pub fn parse_android_strings_xml_files(
    android_project_res_dir: impl AsRef<Path>,
) -> Result<Vec<ParsedStringsXml>> {
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
            Some(other) => {
                if other.starts_with(VALUES_DIR_NAME_WITH_LOCALE_PREFIX) {
                    other.trim_start_matches(VALUES_DIR_NAME_WITH_LOCALE_PREFIX)
                } else {
                    continue;
                }
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
            xml::reader::XmlEvent::StartElement {
                name, attributes, ..
            } => match state {
                ParserState::Start => {
                    if name.local_name == STRINGS_XML_RESOURCES_TAG {
                        state = ParserState::InResources;
                    }
                }
                ParserState::InResources => {
                    if name.local_name == STRINGS_XML_STRING_TAG {
                        state = ParserState::InString;
                        let mut string_value = StringValue {
                            key: String::new(),
                            value: ParsedStringXmlValue::default(),
                            description: None,
                        };
                        handle_string_tag_attributes(&mut string_value, &attributes)?;
                        current_string = Some(string_value);
                    }
                }
                ParserState::InString => (),
            },
            xml::reader::XmlEvent::EndElement { name } => match state {
                ParserState::Start | ParserState::InResources => (),
                ParserState::InString => {
                    if name.local_name == STRINGS_XML_STRING_TAG {
                        state = ParserState::InResources;
                    }
                }
            },
            xml::reader::XmlEvent::Characters(s) => match state {
                ParserState::Start | ParserState::InResources => (),
                ParserState::InString => {
                    if let Some(mut current_string) = current_string.take() {
                        current_string.value = ParsedStringXmlValue::parse(s)?;
                        parsed_xml.strings.push(current_string);
                    }
                }
            },
            _ => {}
        }
    }
    Ok(parsed_xml)
}

fn handle_string_tag_attributes(value: &mut StringValue, tags: &[OwnedAttribute]) -> Result<()> {
    let name_tag = tags
        .iter()
        .find(|a| a.name.local_name == STRINGS_XML_NAME_ATTRIBUTE)
        .ok_or(anyhow!("Missing '{STRINGS_XML_NAME_ATTRIBUTE}' attribute"))?;
    value.key = name_tag.value.clone();

    let description = tags
        .iter()
        .find(|a| a.name.local_name == STRINGS_XML_DESCRIPTION_ATTRIBUTE);
    if let Some(description) = description {
        value.description = Some(description.value.clone());
    }
    Ok(())
}

#[derive(Debug)]
pub enum FormatSpecifierType {
    String,
}

#[derive(Debug)]
pub struct FormatSpecifier {
    pub specifier_type: FormatSpecifierType,
    pub arg_number: u32,
}

impl FormatSpecifier {
    fn to_arb_string(&self) -> String {
        format!("{{{}}}", self.to_arb_placeholder_name())
    }

    pub fn to_arb_placeholder_name(&self) -> String {
        format!("p{}", self.arg_number)
    }

    pub fn to_arb_placeholder_type(&self) -> &'static str {
        match self.specifier_type {
            FormatSpecifierType::String => "String",
        }
    }
}

#[derive(Debug)]
pub enum ParsedStringXmlValuePart {
    FormatSpecifier(FormatSpecifier),
    Text(String),
}

impl ParsedStringXmlValuePart {
    pub fn to_arb_string(&self) -> String {
        match self {
            Self::FormatSpecifier(specifier) => specifier.to_arb_string(),
            Self::Text(t) => t.clone(),
        }
    }

    pub fn is_format_specifier(&self) -> bool {
        matches!(self, Self::FormatSpecifier { .. })
    }
}

impl From<FormatSpecifier> for ParsedStringXmlValuePart {
    fn from(value: FormatSpecifier) -> Self {
        Self::FormatSpecifier(value)
    }
}

impl From<String> for ParsedStringXmlValuePart {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl <'a> From<&'a str> for ParsedStringXmlValuePart {
    fn from(value: &'a str) -> Self {
        Self::Text(value.to_string())
    }
}

#[derive(Debug, Default)]
pub struct ParsedStringXmlValue {
    contents: Vec<ParsedStringXmlValuePart>,
}

impl ParsedStringXmlValue {
    fn parse(text: String) -> Result<Self> {
        let mut data = text.chars();
        let mut current_text = String::new();
        let mut current_arg_number = 0;
        let mut parsed = vec![];

        while let Some(c) = data.next() {
            match c {
                '\\' => {
                    Self::current_to_text_if_needed(&mut current_text, &mut parsed);
                    let success = Self::parse_escaped(data.as_str())?;
                    parsed.push(success.parsed);
                    data = success.new_iterator_state;
                }
                '%' => {
                    Self::current_to_text_if_needed(&mut current_text, &mut parsed);
                    let success =
                        Self::parse_formatting_specifier(data.as_str(), &mut current_arg_number)?;
                    parsed.push(success.parsed);
                    data = success.new_iterator_state;
                }
                c => current_text.push(c),
            }
        }

        Self::current_to_text_if_needed(&mut current_text, &mut parsed);

        Ok(Self { contents: parsed })
    }

    fn parse_escaped(remaining: &str) -> Result<ParseSuccessful> {
        let parsed = if remaining.starts_with('\\') {
            "\\"
        } else if remaining.starts_with('\'') {
            "'"
        } else if remaining.starts_with('\"') {
            "\""
        } else if remaining.starts_with('n') {
            "\n"
        } else if remaining.starts_with('t') {
            "\t"
        } else if remaining.starts_with('@') {
            "@"
        } else if remaining.starts_with('?') {
            "?"
        } else {
            return Err(anyhow!(
                "Parsing escape character failed, remaining: {remaining}"
            ));
        };

        Ok(ParseSuccessful::skip_one_char(remaining, parsed))
    }

    fn parse_formatting_specifier<'a>(
        remaining: &'a str,
        current_arg_number: &mut u32,
    ) -> Result<ParseSuccessful<'a>> {
        if remaining.starts_with('%') {
            Ok(ParseSuccessful::skip_one_char(remaining, "%"))
        } else if remaining.starts_with('s') {
            let parsed = ParseSuccessful::skip_one_char(
                remaining,
                FormatSpecifier {
                    specifier_type: FormatSpecifierType::String,
                    arg_number: *current_arg_number,
                },
            );
            *current_arg_number += 1;
            Ok(parsed)
        } else {
            Err(anyhow!(
                "Parsing formatting specifier failed, remaining: {remaining}"
            ))
        }
    }

    fn current_to_text_if_needed(current: &mut String, parts: &mut Vec<ParsedStringXmlValuePart>) {
        if !current.is_empty() {
            parts.push(ParsedStringXmlValuePart::Text(current.clone()));
            current.clear();
        }
    }

    pub fn to_arb_string(&self) -> String {
        self.contents.iter().map(|v| v.to_arb_string()).collect()
    }

    pub fn format_specifiers(&self) -> impl Iterator<Item = &FormatSpecifier> {
        self.contents.iter().filter_map(|v| {
            if let ParsedStringXmlValuePart::FormatSpecifier(specifier) = v {
                Some(specifier)
            } else {
                None
            }
        })
    }
}

struct ParseSuccessful<'a> {
    new_iterator_state: Chars<'a>,
    parsed: ParsedStringXmlValuePart,
}

impl<'a> ParseSuccessful<'a> {
    pub fn skip_one_char(
        remaining: &'a str,
        parsed: impl Into<ParsedStringXmlValuePart>,
    ) -> ParseSuccessful<'a> {
        let mut new_chars = remaining.chars();
        new_chars.next();
        Self {
            new_iterator_state: new_chars,
            parsed: parsed.into(),
        }
    }
}
