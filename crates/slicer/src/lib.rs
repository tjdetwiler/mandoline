#[cfg(test)]
mod tests {
    use mandoline_mesh::DefaultMesh;

    const STL_CUBE: &[u8] = include_bytes!("../../../res/cube/cube-bin.stl");

    #[test]
    fn create_stl_mesh() {
        let _mesh = mandoline_stl::parse_stl::<DefaultMesh>(STL_CUBE).unwrap();
    }
}
