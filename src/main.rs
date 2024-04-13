mod compiler;
mod dgrp;
mod iff;
mod objd;
mod palt;
mod slot;
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
    Compile { xml_file_path: std::path::PathBuf },
}

fn main() {
    use clap::Parser;
    let cli = Cli::parse();

    match &cli.command {
        CliCommands::Compile { xml_file_path } => {
            compiler::compile(xml_file_path);
        }
    }
}
