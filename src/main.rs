mod compiler;
mod dgrp;
mod iff;
mod iff_description;
mod objd;
mod palt;
mod quantizer;
mod slot;
mod splitter;
mod spr;
mod sprite;
mod the_sims;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
}

#[derive(clap::Subcommand)]
enum CliCommands {
    Split {
        full_sprites_directory: std::path::PathBuf,
        split_sprites_directory: std::path::PathBuf,
        object_dimension_x: i32,
        object_dimension_y: i32,
        frame_names: Vec<String>,
    },
    Compile {
        xml_file_path: std::path::PathBuf,
    },
    CompileAdvanced {
        source_directory: std::path::PathBuf,
        format_string: String,
        creator_name: String,
        object_name: String,
        #[arg(requires_all=["variant_new"])]
        variant_original: Option<String>,
        variant_new: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    use clap::Parser;
    let cli = Cli::parse();

    match &cli.command {
        CliCommands::Split {
            full_sprites_directory,
            split_sprites_directory,
            object_dimension_x,
            object_dimension_y,
            frame_names,
        } => {
            splitter::split(
                full_sprites_directory,
                split_sprites_directory,
                (*object_dimension_x, *object_dimension_y),
                frame_names,
            );
        }
        CliCommands::Compile { xml_file_path } => {
            compiler::compile(xml_file_path)?;
        }
        CliCommands::CompileAdvanced {
            source_directory,
            format_string,
            creator_name,
            object_name,
            variant_original,
            variant_new,
        } => {
            compiler::compile_advanced(
                source_directory,
                format_string,
                creator_name,
                object_name,
                variant_original.as_deref().zip(variant_new.as_deref()),
            )?;
        }
    }
    Ok(())
}
