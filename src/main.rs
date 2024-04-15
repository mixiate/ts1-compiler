mod compiler;
mod dgrp;
mod iff;
mod objd;
mod palt;
mod slot;
mod splitter;
mod spr;
mod sprite;
mod the_sims;
mod xml;

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
    },
    Compile {
        xml_file_path: std::path::PathBuf,
    },
}

fn main() {
    use clap::Parser;
    let cli = Cli::parse();

    match &cli.command {
        CliCommands::Split {
            full_sprites_directory,
            split_sprites_directory,
        } => {
            splitter::split(full_sprites_directory, split_sprites_directory);
        }
        CliCommands::Compile { xml_file_path } => {
            compiler::compile(xml_file_path);
        }
    }
}
