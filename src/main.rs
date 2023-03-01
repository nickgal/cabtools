use std::path::PathBuf;

use cabextract_ce::{list_files, read_cab, CECabinet};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cabextract-ce")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Ls { path: PathBuf },
    Extract {
        path: PathBuf,
        #[arg(last = true)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Ls { path } => {
            println!("Ls {:?}", path);
        }
        Commands::Extract { path, output } => {
            let mut cabinet = read_cab(path);

            let ce_manifest = cabinet.read_000_manifest();
            let file_entries = list_files(&mut cabinet, &ce_manifest);
            let out = match output {
                Some(o) => o,
                _ => PathBuf::from(&ce_manifest.app_name.to_string()),
            };

            cabinet.extract_files(&file_entries, out);

            println!("Done extracting {}", ce_manifest.app_name);
        }
    }
}
