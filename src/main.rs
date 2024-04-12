mod dgrp;
mod iff;
mod objd;
mod slot;
mod spr;
mod xml;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input_xml_file_path = &args[1];
    let output_xml_file_path = &args[2];

    let xml = std::fs::read_to_string(input_xml_file_path).unwrap();

    let iff_xml = quick_xml::de::from_str::<xml::IffXml>(&xml).unwrap();

    let xml_header = include_str!("../res/header.xml");

    let mut buffer = xml_header.to_owned();
    let mut ser =
        quick_xml::se::Serializer::with_root(&mut buffer, Some("objectsexportedfromthesims"))
            .unwrap();
    ser.indent(' ', 2);
    use serde::Serialize;
    iff_xml.serialize(ser).unwrap();

    std::fs::write(output_xml_file_path, &buffer).unwrap();
}
