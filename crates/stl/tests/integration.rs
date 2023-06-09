use mandoline_mesh::{DefaultMesh, TriangleMesh};

const STL_CUBE: &[u8] = include_bytes!("../../../res/cube/cube-bin.stl");

#[test]
#[wasm_bindgen_test::wasm_bindgen_test]
fn parse_cube() {
    let stl = mandoline_stl::parse_stl::<DefaultMesh>(STL_CUBE).unwrap();
    // Expect 12 triangles (2 per face x 6 faces)
    assert_eq!(12, stl.triangle_count());
}
