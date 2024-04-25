use crate::iff;
use crate::the_sims;
use crate::xml;

fn get_iff_file_name_hash(object_name: &str, variant_name: &str) -> String {
    let iff_file_name = format!("{object_name} {variant_name}");
    let iff_file_name_elements: Vec<_> = iff_file_name.split(' ').collect();

    let mut abbreviated_file_name = "".to_owned();
    for element in &iff_file_name_elements {
        if element.len() > 1 {
            abbreviated_file_name.extend(element.chars().take(2));
        }
    }

    use std::hash::Hash;
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    iff_file_name.hash(&mut hasher);

    format!("{abbreviated_file_name}{:X}", hasher.finish() as u32)
}

fn get_formatted_iff_file_path_and_rename_unhashed_iff_file(
    the_sims_downloads_path: &std::path::Path,
    format_string: &str,
    creator_name: &str,
    object_name: &str,
    variant_name: Option<&str>,
) -> anyhow::Result<std::path::PathBuf> {
    use anyhow::Context;
    use formatx::formatx;

    let variant_name = variant_name.unwrap_or("");
    let iff_file_hash = get_iff_file_name_hash(object_name, variant_name);
    let iff_file_name = formatx!(
        format_string,
        name = creator_name,
        hash = iff_file_hash,
        object = object_name,
        variant = variant_name
    )
    .context("Failed to format iff file name")?;
    let iff_file_path = the_sims_downloads_path.join(iff_file_name).with_extension("iff");
    if !iff_file_path.is_file() {
        let unhashed_name = formatx!(
            format_string,
            name = creator_name,
            hash = "",
            object = object_name,
            variant = variant_name
        )
        .context("Failed to format iff file name")?;
        let unhashed_path = the_sims_downloads_path.join(unhashed_name).with_extension("iff");
        std::fs::rename(&unhashed_path, &iff_file_path).with_context(|| {
            format!(
                "Failed to rename {} to {}",
                unhashed_path.display(),
                iff_file_path.display()
            )
        })?;
    }
    Ok(iff_file_path)
}

pub fn compile(xml_file_path: &std::path::Path) -> anyhow::Result<()> {
    use anyhow::Context;

    let mut iff_description = xml::read_xml_file(xml_file_path)?;

    let source_directory = std::path::PathBuf::from(&xml_file_path);
    let source_directory = source_directory.parent().with_context(|| {
        format!(
            "Failed to get source directory from xml file path {}",
            source_directory.display()
        )
    })?;
    iff_description.update_sprite_positions(source_directory)?;

    let the_sims_install_path = the_sims::install_path()?;
    let input_iff_file_path =
        the_sims_install_path.clone().join(&iff_description.iff_file_path_relative).with_extension("iff");

    iff::rebuild_iff_file(
        source_directory,
        &iff_description,
        &input_iff_file_path,
        &input_iff_file_path,
    )?;

    xml::save_xml_file(xml_file_path, &iff_description)?;

    Ok(())
}

pub fn compile_advanced(
    source_directory: &std::path::Path,
    format_string: &str,
    creator_name: &str,
    object_name: &str,
    variant_names: Option<(&str, &str)>,
) -> anyhow::Result<()> {
    let xml_file_path = source_directory.join(object_name).with_extension("xml");

    let mut iff_description = xml::read_xml_file(&xml_file_path)?;

    if let Some((variant_original, variant_new)) = variant_names {
        iff_description.update_sprite_variants(variant_original, variant_new)?;
    }
    iff_description.update_sprite_positions(source_directory)?;

    let (variant_original, variant_new) = variant_names.unzip();
    let the_sims_downloads_path = the_sims::install_path()?.join("downloads");
    let input_iff_file_path = get_formatted_iff_file_path_and_rename_unhashed_iff_file(
        &the_sims_downloads_path,
        format_string,
        creator_name,
        object_name,
        variant_original,
    )?;
    let output_iff_file_path = get_formatted_iff_file_path_and_rename_unhashed_iff_file(
        &the_sims_downloads_path,
        format_string,
        creator_name,
        object_name,
        variant_new,
    )?;

    iff::rebuild_iff_file(
        source_directory,
        &iff_description,
        &input_iff_file_path,
        &output_iff_file_path,
    )?;

    if variant_original == variant_new {
        xml::save_xml_file(&xml_file_path, &iff_description)?;
    }
    Ok(())
}
