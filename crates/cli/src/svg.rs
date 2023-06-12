use std::{collections::HashMap, fs::File, io::Write, path::Path};

use mandoline::{slice_mesh, OrderedVec2, SlicerConfig};
use mandoline_mesh::DefaultMesh;

use crate::args;

pub fn svg_command(args: args::SvgArgs) {
    let config = SlicerConfig { layer_height: 0.2 };
    let mesh = mandoline_stl::read_stl::<DefaultMesh, _>(args.stl_path).unwrap();
    let slices = slice_mesh(mesh, &config);
    generate_svg(args.output, &slices);
}

fn generate_svg<P: AsRef<Path>>(p: P, slices: &Vec<HashMap<OrderedVec2, OrderedVec2>>) {
    let mut f = File::create(p).unwrap();
    writeln!(f, "<svg xmlns=\"http://www.w3.org/2000/svg\" >").unwrap();
    writeln!(f, "  <rect width=\"100%\" height=\"100%\" fill=\"white\"/>").unwrap();
    writeln!(f, "  <defs>").unwrap();
    writeln!(f, "    <marker id=\"arrowhead\" markerWidth=\"6\" markerHeight=\"6\" refX=\"3\" refY=\"1.5\" orient=\"auto\">").unwrap();
    writeln!(
        f,
        "      <polygon points=\"0 0, 2 1.5, 0 3\" fill=\"black\" />"
    )
    .unwrap();
    writeln!(f, "    </marker>").unwrap();
    writeln!(f, "  </defs>").unwrap();
    writeln!(f, "  <g transform=\"translate(0, 0)\">").unwrap();
    let mut values = Vec::new();
    values.resize(slices.len(), "hidden");
    for (layer, segments) in slices.iter().enumerate() {
        writeln!(f, "    <!-- Layer {} -->", layer).unwrap();
        writeln!(f, "    <g id=\"frame{}\">", layer).unwrap();
        writeln!(f, "       <text x=\"10\" y=\"10\">Layer {}</text>", layer).unwrap();
        writeln!(f, "       <g transform=\"translate(15, 20) scale(5)\">").unwrap();
        for (p0, p1) in segments {
            writeln!(f, "        <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#000\" stroke-width=\"0.5\" marker-end=\"url(#arrowhead)\"/>", p0.x, p0.y, p1.x, p1.y).unwrap();
            writeln!(
                f,
                "        <circle cx=\"{}\" cy=\"{}\" r=\"0.5\" />",
                p0.x, p0.y
            )
            .unwrap();
        }
        writeln!(f, "     </g>").unwrap();
        values[layer] = "visible";
        let v = values.join("; ");
        values[layer] = "hidden";
        writeln!(f, "      <animate attributeName=\"visibility\" values=\"{}\" dur=\"5s\" repeatCount=\"indefinite\" />", v).unwrap();
        writeln!(f, "    </g>").unwrap();
    }
    writeln!(f, "  </g>").unwrap();
    writeln!(f, "</svg>").unwrap();
}
