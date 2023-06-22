use std::collections::HashMap;

use cgmath::{InnerSpace, Vector2};
use mandoline_mesh::{Triangle, TriangleMesh, Vector3};
use ordered_float::OrderedFloat;

use crate::config::*;
use crate::contour::*;

pub type OrderedVec2 = Vector2<OrderedFloat<f32>>;
pub type SegmentMap = HashMap<OrderedVec2, OrderedVec2>;

#[inline(always)]
fn float_eq(f1: f32, f2: f32) -> bool {
    float_eq::float_eq!(f1, f2, abs <= 0.0001)
}

trait Truncate {
    fn truncate_micros(self) -> Self;
}

impl Truncate for f32 {
    fn truncate_micros(self) -> Self {
        (self * 1_000.0).round() / 1_000.0
    }
}

pub struct SlicedMesh {
    contours: Vec<Contour>,
    limits_x: (f32, f32),
    limits_y: (f32, f32),
}

impl SlicedMesh {
    pub fn new() -> Self {
        Self {
            contours: Vec::new(),
            limits_x: (0., 0.),
            limits_y: (0., 0.),
        }
    }

    pub fn contours(&self) -> &[Contour] {
        self.contours.as_slice()
    }

    pub fn limits_x(&self) -> (f32, f32) {
        self.limits_x
    }
}

impl Default for SlicedMesh {
    fn default() -> Self {
        Self::new()
    }
}

fn f32_cmp(a: &f32, b: &f32) -> std::cmp::Ordering {
    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
}

fn is_on_plane(p: &Vector3, z: f32) -> bool {
    float_eq(p.z, z)
}

fn intersect(p0: &Vector3, p1: &Vector3, z: f32) -> Option<Vector3> {
    // Return none if no intersection.
    let d0 = p0.z - z;
    let d1 = p1.z - z;

    let e0 = float_eq(p0.z, z);
    let e1 = float_eq(p1.z, z);
    if !e0
        && !e1
        && ((d0.is_sign_negative() && d1.is_sign_negative())
            || (d0.is_sign_positive() && d1.is_sign_positive()))
    {
        return None;
    }

    let t = (z - p0.z) / (p1.z - p0.z);
    Some(Vector3 {
        x: p0.x + ((p1.x - p0.x) * t),
        y: p0.y + ((p1.y - p0.y) * t),
        z,
    })
}

fn compute_min_max(t: &Triangle) -> (f32, f32) {
    // The first step is to compute the slices (layers) that this triangle
    // intersects with. We can do this simply by finding the min and max of
    // the z coordinate since we will define the cutting plane along the z
    // axis.
    let z0 = t.p0.z;
    let z1 = t.p1.z;
    let z2 = t.p2.z;

    let zmax = std::cmp::max_by(std::cmp::max_by(z0, z1, f32_cmp), z2, f32_cmp);
    // Align zmin to the layer height so that all our z coordinates for a slice lie
    // on the cutting plane.
    //
    // TODO: This just rounds to the next lowest layer. We can probably be smarter
    // here.
    let zmin = std::cmp::min_by(std::cmp::min_by(z0, z1, f32_cmp), z2, f32_cmp);

    (zmin, zmax)
}

// Computes the layer numbers that the triangle instersects.
//
// The returned range is the set of cutting planes (defined by multiples of
// layer height) that will intersect the range zmin-zmax. Returns None if
// this triangle does not intersect any cutting planes.
fn compute_constant_layer_range(
    zmin: f32,
    zmax: f32,
    config: &SlicerConfig,
) -> Option<std::ops::RangeInclusive<usize>> {
    let min_layer = (zmin / config.layer_height).ceil() as usize;
    let max_layer = (zmax / config.layer_height).floor() as usize;
    if max_layer > min_layer {
        Some(min_layer..=max_layer)
    } else {
        None
    }
}

/// Given a triangle mesh, we slice it into a series of contour layers using
/// the parameters in `SlicerConfig`.
pub fn slice_mesh<M: TriangleMesh>(m: M, config: &SlicerConfig) -> SlicedMesh {
    // The vector has an entry for each slice, in-order.
    //
    // Each layer is a hash-map that the start of a line segment to the
    // end of that same line segment.
    //
    // This is used to piece the geometry back together at the end.
    //
    // TODO: If triangle mesh knows it's min/max z we can pre-allocate
    // the entire vec here.
    // TODO: HashMap here is not great since we may have rouding errors.
    // We do some course (to nearest um) rouding to mitigate this.
    let mut slices: Vec<HashMap<OrderedVec2, OrderedVec2>> = Vec::new();

    let mut add_slice = |layer, first: &Vector3, second: &Vector3| {
        if slices.len() <= layer {
            slices.resize_with(layer + 1, HashMap::new);
        }

        // Floats are not hash nor eq, so we use the ordered-float crate. This is relying
        // on numeric representations to be identical which is a bit dicey.
        slices[layer].insert(
            Vector2 {
                x: first.x.truncate_micros().into(),
                y: first.y.truncate_micros().into(),
                // z: implicit based on `layer`.
            },
            Vector2 {
                x: second.x.truncate_micros().into(),
                y: second.y.truncate_micros().into(),
                // z: implicit based on `layer`.
            },
        );
    };

    // For each triangle, compute the slices that intersects this triangle
    // and where.
    for t in m.triangles() {
        let (zmin, zmax) = compute_min_max(&t);
        let Some(layer_range) = compute_constant_layer_range(zmin, zmax, config) else {
            continue;
        };
        for layer in layer_range {
            let cutting_plane = layer as f32 * config.layer_height;

            let a_planar = is_on_plane(&t.p0, cutting_plane);
            let b_planar = is_on_plane(&t.p1, cutting_plane);
            let c_planar = is_on_plane(&t.p2, cutting_plane);

            match (a_planar, b_planar, c_planar) {
                // All points lie on the cutting plane. This means the entire triangle
                // is on the cutting plane. We don't generate line segments for this case
                // but instead will generate these line segments from adjacent geometry.
                (true, true, true) => (),

                // If a single point lies on the cutting plane, we also ignore the point.
                //
                // Note we do need to handle the case where the cutting plane intersects a
                // line and a vertex. We know that does not happen if the vertex lies at zmin
                // or zmax.
                (true, false, false) if float_eq(t.p0.z, zmin) || float_eq(t.p0.z, zmax) => (),
                (false, true, false) if float_eq(t.p1.z, zmin) || float_eq(t.p1.z, zmax) => (),
                (false, false, true) if float_eq(t.p2.z, zmin) || float_eq(t.p2.z, zmax) => (),

                // If two points lie on the cutting plane, then one triangle edge
                // represents a line segment to be contributed to the slice.
                (true, true, false) => add_slice(layer, &t.p0, &t.p1),
                (false, true, true) => add_slice(layer, &t.p1, &t.p2),
                (true, false, true) => add_slice(layer, &t.p2, &t.p0),

                // We need to calculate the intersection between the cutting plane and
                // at least one edge. The second intersection will either be another
                // triangle edge, or a triangle vertex.
                _ => {
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
                    // TODO: We need to handle the case of a line-vertex intersection. This
                    // doesn't occur in the first cube model I'm using.
                    if count != 2 {
                        continue;
                    }
                    assert_eq!(count, 2);
                    let (first, second) = if let Some(ab) = ab {
                        (ab, if let Some(bc) = bc { bc } else { ca.unwrap() })
                    } else {
                        (bc.unwrap(), ca.unwrap())
                    };

                    // Direction: We have a triangle with vertices in ccw order, and 2 points
                    // where the slicing plane cuts the trigangle. We need to determine if the
                    // produced vector is first->second or second->first.
                    //
                    // One way to do this is to combine the plane normal with the triangle
                    // normal with a cross product to the the direction vector.
                    let plane_normal = Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    };
                    // u,v are two edge vectors of the triangle. Take their cross product to
                    // find the outward normal vector for this triangle.
                    let u = t.p1 - t.p0;
                    let v = t.p2 - t.p0;
                    let triangle_normal = u.cross(v).normalize();

                    // The direction of the generate line segment is represented by the cross
                    // product of the slicing plane normal and the triangle normal.
                    let direction = plane_normal.cross(triangle_normal).normalize();

                    // Generate the line segment that is in the same direction we expect.
                    let forward = first - second;
                    if forward.dot(direction) > 0.0 {
                        add_slice(layer, &first, &second);
                    } else {
                        add_slice(layer, &second, &first);
                    }
                }
            }
        }
    }
    slices.into_iter().fold(SlicedMesh::new(), |mut a, x| {
        let c = Contour::from_segment_map(x);
        let xlim = c.limits_x();
        let ylim = c.limits_y();
        if xlim.0 < a.limits_x.0 {
            a.limits_x.0 = xlim.0;
        }
        if xlim.1 > a.limits_x.1 {
            a.limits_x.1 = xlim.1;
        }
        if ylim.0 < a.limits_y.0 {
            a.limits_y.0 = ylim.0;
        }
        if ylim.1 > a.limits_y.1 {
            a.limits_y.1 = ylim.1;
        }
        a.contours.push(c);
        a
    })
}

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;
    use mandoline_test_data::{STL_CALIBRATION_CUBE, STL_CUBE};

    use {super::*, mandoline_mesh::DefaultMesh};

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

    // intersect with a line parallel to the plane.
    //
    // The slicer should detect this situation and not call intersect
    // on these points.
    #[test]
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
        assert!(intersection.x.is_nan());
        assert!(intersection.y.is_nan());
        assert_eq!(intersection.z, 0.0);
    }

    #[test]
    fn slice_simple_cube() {
        let config = SlicerConfig { layer_height: 0.2 };
        let mesh = mandoline_stl::parse_stl::<DefaultMesh>(STL_CUBE.bytes).unwrap();
        slice_mesh(mesh, &config);
    }

    #[test]
    fn slice_calibration_cube() {
        let config = SlicerConfig { layer_height: 0.2 };
        let mesh = mandoline_stl::parse_stl::<DefaultMesh>(STL_CALIBRATION_CUBE.bytes).unwrap();
        slice_mesh(mesh, &config);
    }
}
