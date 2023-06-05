use mesh::VertexIndex;

const STL_CUBE: &[u8] = include_bytes!("../../../res/cube/cube-bin.stl");

#[test]
fn create_cube_mesh() {
    let _mesh = stl_loader::parse_stl::<VertexIndex>(STL_CUBE).unwrap();
}
