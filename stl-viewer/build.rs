use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;

fn main() -> Result<(), anyhow::Error> {
    // This tells cargo to rerun this script if something in /res/ changes.
    println!("cargo:rerun-if-changed=res/*");

    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = Vec::new();
    paths_to_copy.push("res/");
    copy_items(&paths_to_copy, "pkg", &copy_options)?;
    Ok(())
}
