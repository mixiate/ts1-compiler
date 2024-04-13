use crate::iff;
use crate::the_sims;
use crate::xml;

pub fn compile(xml_file_path: &std::path::Path) {
    let xml = std::fs::read_to_string(xml_file_path).unwrap();

    let mut iff_description = quick_xml::de::from_str::<xml::IffDescription>(&xml).unwrap();

    let source_directory = std::path::PathBuf::from(&xml_file_path);
    let source_directory = source_directory.parent().unwrap();
    iff_description.update_sprite_positions(source_directory);

    let the_sims_install_path = the_sims::install_path();
    let input_iff_file_path =
        the_sims_install_path.clone().join(&iff_description.iff_file_path_relative).with_extension("iff");

    iff::rebuild_iff_file(
        source_directory,
        &iff_description,
        &input_iff_file_path,
        &input_iff_file_path,
    );

    let xml_header = include_str!("../res/header.xml");

    let mut buffer = xml_header.to_owned();
    let mut ser = quick_xml::se::Serializer::with_root(&mut buffer, Some("objectsexportedfromthesims")).unwrap();
    ser.indent(' ', 2);
    use serde::Serialize;
    iff_description.serialize(ser).unwrap();

    std::fs::write(xml_file_path, &buffer).unwrap();
}
