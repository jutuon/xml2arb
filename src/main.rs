pub mod arb;
pub mod config;
pub mod xml;

fn main() {
    let config = config::get_config();

    let parsed_strings = match xml::parse_android_strings_xml_files(&config.input_dir) {
        Ok(parsed_strings) => parsed_strings,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = arb::save_to_arb_files(
        &config.output_dir,
        parsed_strings,
        config.arb_file_name_template,
    ) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
