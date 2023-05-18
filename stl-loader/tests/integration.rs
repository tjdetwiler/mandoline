use std::path::{Path, PathBuf};

fn model_path<P: AsRef<Path>>(path: P) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(path)
}

#[test]
fn load_cube() {
    let stl = stl_loader::read_stl(model_path("models/cube/cube-bin.stl"))
        .expect("failed to read STL file");
    // Expect 12 triangles (2 per face x 6 faces)
    assert_eq!(12, stl.triangles.len());
    println!("{:#?}", stl.triangles);
}
