use std::collections::HashMap;

use mandoline_mesh::{Triangle, TriangleMesh, Vector3};

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

// Computes the layer numbers that the triangle instersects.
//
// This assumes a constant layer height as defined by the `config`.
fn compute_constant_layer_range(t: &Triangle, config: &SlicerConfig) -> std::ops::Range<usize> {
    // The first step is to compute the slices (layers) that this triangle
    // intersects with. We can do this simply by finding the min and max of
    // the z coordinate since we will define the cutting plane along the z
    // axis.
    let z0 = t.p0.z as f64;
    let z1 = t.p1.z as f64;
    let z2 = t.p2.z as f64;

    let zmax = std::cmp::max_by(std::cmp::max_by(z0, z1, f32_cmp), z2, f32_cmp);
    // Align zmin to the layer height so that all our z coordinates for a slice lie
    // on the cutting plane.
    //
    // TODO: This just rounds to the next lowest layer. We can probably be smarter
    // here.
    let zmin = std::cmp::min_by(std::cmp::min_by(z0, z1, f32_cmp), z2, f32_cmp);

    let min_layer = (zmin / config.layer_height).round() as usize;
    let max_layer = zmax / config.layer_height;
    let max_layer = (max_layer + 1.0).round() as usize;

    min_layer..max_layer
}

pub fn slice_mesh<M: TriangleMesh>(m: M, config: &SlicerConfig) {
    // The vector has an entry for each slice, in-order.
    //
    // Each layer is a hash-map that the start of a line segment to the
    // end of that same line segment.
    //
    // This is used to piece the geometry back together at the end.
    //
    // TODO: If triangle mesh knows it's min/max z we can pre-allocate
    // the entire vec here.
    let mut slices: Vec<HashMap<Vector3, Vector3>> = Vec::new();

    // For each triangle, compute the slices that intersects this triangle
    // and where.
    for t in m.triangles() {
        for layer in compute_constant_layer_range(&t, config) {
            let cutting_plane = (layer as f64 * config.layer_height) as f32;
            // Compute intersection points.
            //
            // We have 3 points that define a triangle, and a cutting plane that is
            // defined by the normal vector that lies along +z and the distance of
            // the cutting plane from the origin in the variable `cutting_plane`.
            //
            // If we label the points of the triangle as a, b, c such that these
            // points occur in a counter-clockwise when looking at the front of the
            // triangle, we next compute if any of the 3 line segments ab, bc, ca
            // intersect with the cutting plane. Here `None` means no intersection,
            // otherwise the coordinate of the intersection point is provided.
            let ab = intersect(&t.p0, &t.p1, cutting_plane);
            let bc = intersect(&t.p1, &t.p2, cutting_plane);
            let ca = intersect(&t.p2, &t.p0, cutting_plane);

            // Compute the total number of intersection points.
            let mut count = 0;
            for intersection in &[ab, bc, ca] {
                if intersection.is_some() {
                    count += 1;
                }
            }
            // Based on the number of intersection points we have a few possible
            // situations:
            //
            //   * 0 points - This triangle does not intersect the cutting plane. This
            //     should not be possible since we have computed the layers such that
            //     they should intesect the cutting plane.
            assert_ne!(count, 0);
            //   * 1 point  - one verex intersects the cutting plane. This is also not
            //     expected because this situation will be represented by 2 points with
            //     the same coordinate; one from each line segment that touches the
            //     plane.
            assert_ne!(count, 1);
            //   * 3 points - The triangle lies on the cutting plane. For now we skip this
            //     and rely on adjacent triangles to provide these line segments.
            if count == 3 {
                break;
            }
            //   * 2 points - 2 of the line segments cut through the cutting plane,
            //     producing a line segment.
            assert!(count == 2);
            let (_first, _second) = if let Some(ab) = ab {
                (ab, if let Some(bc) = bc { bc } else { ca.unwrap() })
            } else {
                (bc.unwrap(), ca.unwrap())
            };
            if slices.len() <= layer {
                slices.reserve(layer - slices.len());
            }
            // TODO: Floats are not hash nor eq. Perhaps a fixed point number would be preferable here.
            //
            // slices[layer].insert(first, second);
        }
    }
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    use {super::*, mandoline_mesh::DefaultMesh};

    const STL_CUBE: &[u8] = include_bytes!("../../../res/cube/cube-bin.stl");

    #[test]
    fn intersect_no_intersection() {
        // A line below the cutting plane:
        assert_eq!(
            None,
            intersect(
                &Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                &Vector3 {
                    x: 1.0,
                    y: 1.0,
                    z: 0.0,
                },
                0.1
            )
        );
        // A line above the cutting plane:
        assert_eq!(
            None,
            intersect(
                &Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.1,
                },
                &Vector3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.1,
                },
                1.0
            )
        );
    }

    #[test]
    fn intersect_plane_with_line() {
        // Some lines through the cutting plane.

        // 0,0,0 -> 0,0.1 intersects at 0,0,0.5
        let intersection = intersect(
            &Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            &Vector3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            0.5,
        )
        .unwrap();
        assert_float_eq!(intersection.x, 0.0, abs <= 0.0001);
        assert_float_eq!(intersection.y, 0.0, abs <= 0.0001);
        assert_float_eq!(intersection.z, 0.5, abs <= 0.0001);
    }

    // TODO: intersect is currently broken for this situation.
    //
    // We will need to tweak the interface to support this situation.
    #[test]
    #[ignore]
    fn intersect_plane_with_parallel_line() {
        // Line on the cutting plane:
        let intersection = intersect(
            &Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            &Vector3 {
                x: 1.0,
                y: 1.0,
                z: 0.0,
            },
            0.0,
        )
        .unwrap();
        assert!(!intersection.x.is_nan());
        assert!(!intersection.y.is_nan());
        assert_float_eq!(intersection.z, 0.5, abs <= 0.0001);
        println!("intersection: {:?}", intersection);
    }

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
