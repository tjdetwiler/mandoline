use std::{fs::File, io::Write};

use mandoline::{slice_mesh, Contour, SlicerConfig};
use mandoline_mesh::DefaultMesh;

use crate::args;

pub struct SvgCommand {
    args: args::SvgArgs,
    animation_state: Option<Vec<&'static str>>,
}

impl SvgCommand {
    pub fn from_args(args: args::SvgArgs) -> Self {
        let animation_state = if args.layer.is_none() {
            Some(Vec::new())
        } else {
            None
        };
        Self {
            args,
            animation_state,
        }
    }

    pub fn run(mut self) {
        let config = SlicerConfig { layer_height: 0.2 };
        let mesh = mandoline_stl::read_stl::<DefaultMesh, _>(&self.args.stl_path).unwrap();
        let slices = slice_mesh(mesh, &config);

        let layers = self.args.layer.map(|l| l..l + 1).unwrap_or(0..slices.len());
        if let Some(vec) = self.animation_state.as_mut() {
            vec.resize(layers.len(), "hidden");
        }

        let mut f = File::create(&self.args.output).unwrap();
        write_svg_header(&mut f);
        for layer in layers {
            self.generate_layer_paths(&mut f, layer, &slices[layer]);
        }
        write_svg_footer(&mut f);
    }

    fn generate_layer_paths<W: Write>(&mut self, f: &mut W, layer: usize, segments: &Contour) {
        writeln!(f, "    <!-- Layer {} -->", layer).unwrap();
        writeln!(f, "    <g id=\"frame{}\">", layer).unwrap();
        writeln!(f, "       <text x=\"10\" y=\"10\">Layer {}</text>", layer).unwrap();
        writeln!(f, "       <g transform=\"translate(15, 20) scale(5)\">").unwrap();
        for path in segments.paths() {
            for (p0, p1) in path.segments() {
                writeln!(f, "        <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#000\" stroke-width=\"0.5\" marker-end=\"url(#arrowhead)\"/>", p0.0, p0.1, p1.0, p1.1).unwrap();
                writeln!(
                    f,
                    "        <circle cx=\"{}\" cy=\"{}\" r=\"0.5\" />",
                    p0.0, p0.1
                )
                .unwrap();
            }
        }
        writeln!(f, "     </g>").unwrap();
        if let Some(values) = self.animation_state.as_mut() {
            values[layer] = "visible";
            let v = values.join("; ");
            values[layer] = "hidden";
            writeln!(f, "      <animate attributeName=\"visibility\" values=\"{}\" dur=\"5s\" repeatCount=\"indefinite\" />", v).unwrap();
        }
        writeln!(f, "    </g>").unwrap();
    }
}

impl args::Subcommand<args::SvgArgs> for SvgCommand {
    fn run_command(args: args::SvgArgs) {
        Self::from_args(args).run()
    }
}

fn write_svg_header<W: Write>(f: &mut W) {
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
}

fn write_svg_footer<W: Write>(f: &mut W) {
    writeln!(f, "  </g>").unwrap();
    writeln!(f, "</svg>").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::Subcommand;
    use tempfile::NamedTempFile;

    fn is_error_event(e: &svg::parser::Event) -> bool {
        matches!(e, svg::parser::Event::Error(_))
    }

    #[test]
    fn test_animated_svg() {
        // Given - a file with a calibration cube written.
        const CALIBRATION_CUBE: &[u8] =
            include_bytes!("../../../res/calibration-cube/cube-bin.stl");
        let mut input = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();
        input.write_all(CALIBRATION_CUBE).unwrap();
        let args = args::SvgArgs {
            // No layer means to render an svg of all layers.
            layer: None,
            output: output.path().to_str().map(|s| s.to_owned()).unwrap(),
            stl_path: input.path().to_str().map(|s| s.to_owned()).unwrap(),
        };

        // When - Execute the command
        SvgCommand::run_command(args);

        // Then - The output file should be a valid SVG.
        let mut content = String::new();
        let svg_file = svg::open(output.path(), &mut content).expect("Failed to parse svg");

        // The parser is lazy, so verify we can read the entire file.
        //
        // The svg library we use doesn't do much interesting validation, so this
        // isn't validating much.
        let events = svg_file.collect::<Vec<_>>();
        assert!(!events.iter().any(is_error_event));

        // Verify we see the right number of 'Layer N' comments in the file so that
        // it looks like we did generate the right number of images in our animation.
        for evt in events.iter() {
            if let svg::parser::Event::Comment(c) = evt {
                println!("{}", c);
            }
        }
        let layer_comments = events
            .iter()
            .filter(
                |evt| matches!(evt, svg::parser::Event::Comment(c) if c.starts_with("<!-- Layer")),
            )
            .collect::<Vec<_>>();
        // TODO: this should be 100.
        assert_eq!(101, layer_comments.len());
    }
}
