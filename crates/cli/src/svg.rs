use std::{fs::File, io::Write};

use mandoline::{slice_mesh, Contour, SlicerConfig};
use mandoline_mesh::DefaultMesh;

use crate::args;

const DEFAULT_SVG_MARGIN: usize = 15;
const DEFAULT_SVG_CONTENT_WIDTH: usize = 100;
const DEFAULT_SVG_WIDTH: usize = DEFAULT_SVG_CONTENT_WIDTH + 2 * DEFAULT_SVG_MARGIN;

#[derive(Debug)]
enum SvgMode {
    Animated { state: Option<Vec<&'static str>> },
    SingleLayer(usize),
    Grid,
}

#[derive(Debug)]
struct Transform {
    source_offset: f32,
    dest_offset: f32,
    scale: f32,
}

impl Transform {
    pub fn identity() -> Self {
        Transform {
            source_offset: 0.0,
            dest_offset: 0.0,
            scale: 1.0,
        }
    }

    pub fn apply(&self, v: f32) -> f32 {
        v * self.scale + self.dest_offset
    }
}

#[derive(Debug)]
pub struct SvgCommand {
    args: args::SvgArgs,
    mode: SvgMode,
    // Linear transform from contour coordinates to svg pixels.
    transform: Transform,
}

impl SvgCommand {
    pub fn from_args(args: args::SvgArgs) -> Self {
        let mode = match args {
            args::SvgArgs {
                layer: Some(layer), ..
            } => SvgMode::SingleLayer(layer),
            _ if args.grid => SvgMode::Grid,
            _ => SvgMode::Animated {
                state: Some(Vec::new()),
            },
        };
        Self {
            args,
            mode,
            transform: Transform::identity(),
        }
    }

    pub fn run(mut self) {
        let config = SlicerConfig { layer_height: 0.2 };
        let mesh = mandoline_stl::read_stl::<DefaultMesh, _>(&self.args.stl_path).unwrap();
        let slices = slice_mesh(mesh, &config);

        // Update our transform.
        let svg_width = (self.args.frame_width.unwrap_or(DEFAULT_SVG_CONTENT_WIDTH)) as f32;
        let model_width = slices.limits_x().1 - slices.limits_x().0;

        self.transform.dest_offset = 0.0;
        self.transform.source_offset = -slices.limits_x().0;
        self.transform.scale = svg_width / model_width;

        let layers = self
            .args
            .layer
            .map(|l| l..l + 1)
            .unwrap_or(0..slices.contours().len());
        if let SvgMode::Animated { state: Some(vec) } = &mut self.mode {
            vec.resize(layers.len(), "hidden");
        }

        let mut f = File::create(&self.args.output).unwrap();
        write_svg_header(&mut f);
        for layer in layers {
            self.generate_layer_paths(&mut f, layer, &slices.contours()[layer]);
        }
        write_svg_footer(&mut f);
    }

    fn generate_layer_paths<W: Write>(&mut self, f: &mut W, layer: usize, segments: &Contour) {
        let height = self.transform.apply(segments.limits_y().1).ceil() as usize;
        let height = height + 2 * DEFAULT_SVG_MARGIN;
        let frame_pos = if let SvgMode::Grid = self.mode {
            let row = layer / 10;
            let col = layer % 10;
            (col * DEFAULT_SVG_WIDTH, row * height)
        } else {
            (0, 0)
        };
        writeln!(f, "    <!-- Layer {} -->", layer).unwrap();
        writeln!(f, "    <g id=\"frame{}\">", layer).unwrap();
        writeln!(
            f,
            "       <g transform=\"translate({}, {})\">",
            frame_pos.0, frame_pos.1
        )
        .unwrap();
        writeln!(f, "       <text x=\"10\" y=\"10\">Layer {}</text>", layer).unwrap();
        writeln!(
            f,
            "       <g transform=\"translate({}, {})\">",
            DEFAULT_SVG_MARGIN, DEFAULT_SVG_MARGIN
        )
        .unwrap();
        for path in segments.paths() {
            for (p0, p1) in path.segments() {
                writeln!(
                    f,
                    "        <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#000\" stroke-width=\"0.5\" marker-end=\"url(#arrowhead)\"/>",
                    self.transform.apply(p0.0),
                    self.transform.apply(p0.1),
                    self.transform.apply(p1.0),
                    self.transform.apply(p1.1)
                ).unwrap();
                writeln!(
                    f,
                    "        <circle cx=\"{}\" cy=\"{}\" r=\"0.5\" />",
                    self.transform.apply(p0.0),
                    self.transform.apply(p0.1)
                )
                .unwrap();
            }
        }
        writeln!(f, "     </g>").unwrap();
        writeln!(f, "     </g>").unwrap();
        if let SvgMode::Animated {
            state: Some(values),
        } = &mut self.mode
        {
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
    const ARROW_WIDTH: f32 = 5.0;
    const ARROW_HEIGHT: f32 = 10.0;
    writeln!(f, "<svg xmlns=\"http://www.w3.org/2000/svg\" >").unwrap();
    writeln!(f, "  <rect width=\"100%\" height=\"100%\" fill=\"white\"/>").unwrap();
    writeln!(f, "  <defs>").unwrap();
    writeln!(
        f,
        "    <marker id=\"arrowhead\" markerWidth=\"{}\" markerHeight=\"{}\" refX=\"{}\" refY=\"{}\" orient=\"auto\">",
        ARROW_HEIGHT,
        ARROW_WIDTH,
        ARROW_HEIGHT,
        ARROW_WIDTH / 2.0,
    ).unwrap();
    writeln!(
        f,
        "      <polygon points=\"0 0, {} {}, 0 {}\" fill=\"black\" />",
        ARROW_HEIGHT,
        ARROW_WIDTH / 2.0,
        ARROW_WIDTH,
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
            grid: false,
            frame_width: None,
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
