use mandoline_mesh::{TriangleMesh, Vector3};

#[derive(Default)]
#[non_exhaustive]
pub struct SlicerConfig {
    layer_height: f64,
}

fn f32_cmp(a: &f64, b: &f64) -> std::cmp::Ordering {
    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
}

fn intersect(p0: &Vector3, p1: &Vector3, z: f32) -> Option<Vector3> {
    // Return none if no intersection.
    let d0 = p0.z - z;
    let d1 = p1.z - z;

    let e0 = float_eq::float_eq!(p0.z, z, abs <= 0.0001);
    let e1 = float_eq::float_eq!(p1.z, z, abs <= 0.0001);
    if !e0
        && !e1
        && ((d0.is_sign_negative() && d1.is_sign_negative())
            || (d0.is_sign_positive() && d1.is_sign_positive()))
    {
        //println!("no intersect {}<->{} z {} e0 {} e1 {}", d0, d1, z, e0, e1);
        return None;
    }

    let t = (z - p0.z) / (p1.z - p0.z);
    Some(Vector3 {
        x: p0.x + ((p1.x - p0.x) * t),
        y: p0.y + ((p1.y - p0.y) * t),
        z,
    })
}

pub fn slice_mesh<M: TriangleMesh>(m: M, config: &SlicerConfig) {
    // For each triangle, compute the slices that intersects this triangle
    // and where.
    for t in m.triangles() {
        let p0 = t.p0.z as f64;
        let p1 = t.p1.z as f64;
        let p2 = t.p2.z as f64;

        let zmin = std::cmp::min_by(std::cmp::min_by(p0, p1, f32_cmp), p2, f32_cmp);
        let zmax = std::cmp::max_by(std::cmp::max_by(p0, p1, f32_cmp), p2, f32_cmp);

        let mut z = zmin;
        println!("Triangle: {:?}", t);
        while z <= zmax {
            println!("Slicing z = {}", z);
            // Compute intersection points.
            let ab = intersect(&t.p0, &t.p1, z as f32);
            let bc = intersect(&t.p1, &t.p2, z as f32);
            let ca = intersect(&t.p2, &t.p0, z as f32);
            let mut count = 0;
            for intersection in &[ab, bc, ca] {
                if intersection.is_some() {
                    count += 1;
                }
            }
            // We computed only the slices that intersects this triangle, so we never
            // expect 0 here.
            assert_ne!(count, 0);
            // We don't expect 1 here. If we have a touch point on a vertex we still
            // will have 2 points here (one from each line segment that generated the
            // point).
            assert_ne!(count, 1);

            // If all 3 lines intersect the cutting plane, then this triangle lies on
            // the cutting plane itself.
            //
            // TODO: do we ignore these? In theory we will have these edges filled by the
            // adjoining triangles.
            if count == 3 {
                break;
            }

            assert!(count == 2);
            let (first, second) = if ab.is_some() {
                (ab, if bc.is_some() { bc } else { ca })
            } else {
                (bc, ca)
            };
            println!("line {:?} -> {:?}", first, second);
            z += config.layer_height;
        }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, mandoline_mesh::DefaultMesh};

    const STL_CUBE: &[u8] = include_bytes!("../../../res/cube/cube-bin.stl");

    #[test]
    fn create_stl_mesh() {
        let _mesh = mandoline_stl::parse_stl::<DefaultMesh>(STL_CUBE).unwrap();
    }

    #[test]
    fn slice_cube() {
        let config = SlicerConfig {
            layer_height: 1.0,
            ..Default::default()
        };
        let mesh = mandoline_stl::parse_stl::<DefaultMesh>(STL_CUBE).unwrap();
        slice_mesh(mesh, &config);
    }
}
