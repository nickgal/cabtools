use std::path::PathBuf;

use cabtools::{read_cab, CECabinet};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cabtools")]
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
            let mut cabinet = read_cab(path);
            let ce_manifest = cabinet.read_000_manifest();
            let file_entries = cabinet.list_files();

            println!("App name {}", ce_manifest.app_name.to_string());
            println!("{} Files", file_entries.len());
        }
        Commands::Extract { path, output } => {
            let mut cabinet = read_cab(path);

            let ce_manifest = cabinet.read_000_manifest();
            let file_entries = cabinet.list_files();
            let out = match output {
                Some(o) => o,
                _ => PathBuf::from(&ce_manifest.app_name.to_string()),
            };

            cabinet.extract_files(&file_entries, out);

            println!("Done extracting {}", ce_manifest.app_name);
        }
    }
}
