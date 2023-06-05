#[cfg(test)]
mod tests {
    use mesh::VertexIndex;

    const STL_CUBE: &[u8] = include_bytes!("../../../res/cube/cube-bin.stl");

    #[test]
    fn create_stl_mesh() {
        let stl = stl_loader::parse_stl(STL_CUBE).unwrap();
        let _mesh = VertexIndex::from_stl(stl);
    }
}
