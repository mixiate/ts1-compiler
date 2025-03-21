mod compiler;
mod dgrp;
mod draw_group_rotation_adder;
mod error;
mod iff;
mod iff_description;
mod objd;
mod palt;
mod quantizer;
mod slot;
mod splitter;
mod spr;
mod sprite;
mod xml_updater;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
}

#[derive(clap::Subcommand)]
enum CliCommands {
    Split {
        source_directory: std::path::PathBuf,
        object_name: String,
        #[arg(short, long)]
        variant: Option<String>,
    },
    UpdateXml {
        source_directory: std::path::PathBuf,
        object_name: String,
        #[arg(short, long)]
        variant: Option<String>,
    },
    Compile {
        the_sims_directory: std::path::PathBuf,
        xml_file_path: std::path::PathBuf,
    },
    CompileAdvanced {
        the_sims_directory: std::path::PathBuf,
        source_directory: std::path::PathBuf,
        format_string: String,
        creator_name: String,
        object_name: String,
        #[arg(requires_all=["variant_new"])]
        variant_original: Option<String>,
        variant_new: Option<String>,
    },
    AddRotations {
        xml_file_path: std::path::PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    use clap::Parser;
    let cli = Cli::parse();

    match &cli.command {
        CliCommands::Split {
            source_directory,
            object_name,
            variant,
        } => {
            splitter::split(source_directory, object_name, variant.as_deref())?;
        }
        CliCommands::UpdateXml {
            source_directory,
            object_name,
            variant,
        } => {
            xml_updater::update(source_directory, object_name, variant.as_deref())?;
        }
        CliCommands::Compile {
            the_sims_directory,
            xml_file_path,
        } => {
            compiler::compile(the_sims_directory, xml_file_path)?;
        }
        CliCommands::CompileAdvanced {
            the_sims_directory,
            source_directory,
            format_string,
            creator_name,
            object_name,
            variant_original,
            variant_new,
        } => {
            compiler::compile_advanced(
                the_sims_directory,
                source_directory,
                format_string,
                creator_name,
                object_name,
                variant_original.as_deref().zip(variant_new.as_deref()),
            )?;
        }
        CliCommands::AddRotations { xml_file_path } => {
            draw_group_rotation_adder::add_rotations(xml_file_path)?;
        }
    }
    Ok(())
}
