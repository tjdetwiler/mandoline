use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    io::Write,
    path::Path,
};

use cgmath::Vector2;
use ordered_float::OrderedFloat;

use crate::OrderedVec2;

const MUL: f32 = 3.0;

#[inline(always)]
fn node_name(n: Vector2<OrderedFloat<f32>>) -> String {
    let mut s = DefaultHasher::new();
    n.hash(&mut s);
    format!("n{:x}", s.finish())
}

pub fn generate_layer_graph<P: AsRef<Path>>(path: P, segments: &HashMap<OrderedVec2, OrderedVec2>) {
    let mut f = std::fs::File::create(path).unwrap();
    writeln!(f, "digraph G {{").unwrap();

    // Emit nodes
    writeln!(f, "\t{{").unwrap();
    for (p, _) in segments.iter() {
        writeln!(
            f,
            "\t\t{} [fixedsize=true label=\"({},{})\", pos=\"{},{}\"]",
            node_name(*p),
            p.x,
            p.y,
            p.x * MUL,
            p.y * MUL
        )
        .unwrap();
    }
    writeln!(f, "\t}}").unwrap();

    // Emit edges
    let (start, mut next) = segments.iter().next().unwrap();
    writeln!(f, "\t{} -> {{ {} }}", node_name(*start), node_name(*next)).unwrap();
    while next != start {
        if let Some(pt) = segments.get(next) {
            writeln!(f, "\t{} -> {{ {} }}", node_name(*next), node_name(*pt)).unwrap();
            next = pt;
        } else {
            break;
        }
    }

    writeln!(f, "}}").unwrap();
}
