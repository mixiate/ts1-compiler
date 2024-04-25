use crate::iff_description;

pub fn read_xml_file(xml_file_path: &std::path::Path) -> anyhow::Result<iff_description::IffDescription> {
    use anyhow::Context;

    let iff_description = std::fs::read_to_string(xml_file_path)
        .with_context(|| format!("Failed to read xml file {}", xml_file_path.display()))?;
    quick_xml::de::from_str::<iff_description::IffDescription>(&iff_description)
        .with_context(|| format!("Failed to deserialize xml file {}", xml_file_path.display()))
}

pub fn save_xml_file(
    xml_file_path: &std::path::Path,
    iff_description: &iff_description::IffDescription,
) -> anyhow::Result<()> {
    use anyhow::Context;

    let xml_header = include_str!("../res/header.xml");

    let mut buffer = xml_header.to_owned();
    let mut serializer = quick_xml::se::Serializer::with_root(&mut buffer, Some("objectsexportedfromthesims"))
        .context("Failed to serialize xml file")?;
    serializer.indent(' ', 2);
    use serde::Serialize;
    iff_description.serialize(serializer).context("Failed to serialize xml file")?;

    std::fs::write(xml_file_path, &buffer)
        .with_context(|| format!("Failed to write xml to {}", xml_file_path.display()))?;
    Ok(())
}
