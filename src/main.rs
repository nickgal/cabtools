use cabextract_ce::{export_files, list_files, read_cab, CECabinet};

fn main() {
    let mut cabinet = read_cab("PocketEQDemoPPC2002.STRONGARM.cab");

    let ce_manifest = cabinet.read_000_manifest();
    let file_entries = list_files(&mut cabinet, &ce_manifest);
    export_files(&mut cabinet, &file_entries);

    println!("Done extracting {}", ce_manifest.app_name);
}
