
pub mod config;
pub mod arb;
pub mod xml;


fn main() {
    let config = config::get_config();

    let parsed_strings = xml::parse_android_strings_xml_files(&config.input_dir).unwrap();
    arb::save_to_arb_files(&config.output_dir, parsed_strings, config.arb_file_name_template).unwrap();
}
