use mandoline::{slice_mesh, SlicerConfig};
use mandoline_mesh::DefaultMesh;

mod svg;

fn main() {
    const STL_CUBE: &[u8] = include_bytes!("../../../res/calibration-cube/cube-bin.stl");

    let config = SlicerConfig { layer_height: 0.2 };

    let mesh = mandoline_stl::parse_stl::<DefaultMesh>(STL_CUBE).unwrap();
    let slices = slice_mesh(mesh, &config);

    svg::generate_svg("./out.svg", &slices);
}
