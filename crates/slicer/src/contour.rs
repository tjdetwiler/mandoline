use std::collections::HashMap;

use cgmath::Vector2;

use crate::{OrderedVec2, SegmentMap};

pub struct ClosedPath {
    /// Points in the closed path.
    ///
    /// Since this path is closed by definition, the first point is also
    /// implicitly the last point.
    ///
    /// Ex, the triangle A,B,C
    ///
    /// ```text
    ///         A --- B
    ///         |    /
    ///         |   /
    ///         |  /
    ///         | /
    ///         |/
    ///         C
    /// ```
    ///
    /// Would be stored as the Vector: {A, B C}, which implies the path
    /// edges: A->B, B->C, C->A.
    path: Vec<Vector2<f32>>,
}

impl ClosedPath {
    pub fn new() -> Self {
        Self { path: Vec::new() }
    }

    pub fn add_point(&mut self, x: f32, y: f32) {
        self.path.push(Vector2 { x, y })
    }

    pub fn points(&self) -> &[Vector2<f32>] {
        self.path.as_slice()
    }

    pub fn points_vec(&mut self) -> &mut Vec<Vector2<f32>> {
        &mut self.path
    }

    pub fn segments(&self) -> Segments {
        let mut iter = self.path.iter();
        let start = iter.next().cloned();
        Segments {
            prev: start,
            start,
            iter,
        }
    }
}

pub struct Segments<'a> {
    start: Option<Vector2<f32>>,
    prev: Option<Vector2<f32>>,
    iter: std::slice::Iter<'a, Vector2<f32>>,
}

impl<'a> Iterator for Segments<'a> {
    type Item = ((f32, f32), (f32, f32));

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(prev) = self.prev {
            let next = if let Some(next) = self.iter.next().cloned() {
                self.prev = Some(next);
                next
            } else {
                self.prev = None;
                self.start.unwrap()
            };
            Some(((prev.x, prev.y), (next.x, next.y)))
        } else {
            None
        }
    }
}

impl Default for ClosedPath {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Contour {
    paths: Vec<ClosedPath>,
    // The low/high point in this contour.
    limits_x: (f32, f32),
    limits_y: (f32, f32),
}

impl Contour {
    pub fn new() -> Contour {
        Contour {
            paths: Vec::new(),
            limits_x: (0., 0.),
            limits_y: (0., 0.),
        }
    }

    pub fn add_path(&mut self, path: ClosedPath) {
        self.paths.push(path)
    }

    pub fn paths(&self) -> &[ClosedPath] {
        self.paths.as_slice()
    }

    pub fn limits_x(&self) -> (f32, f32) {
        self.limits_x
    }

    pub fn limits_y(&self) -> (f32, f32) {
        self.limits_y
    }
}

impl Default for Contour {
    fn default() -> Self {
        Self::new()
    }
}

fn is_parallel<A: Into<f32>, B: Into<f32>>(v0: Vector2<A>, v1: Vector2<B>) -> bool {
    ((v0.x.into() * v1.y.into()) - (v0.y.into() * v1.x.into())) == 0.0
}

impl Contour {
    pub fn from_segment_map(mut map: SegmentMap) -> Self {
        let mut paths = Contour::new();
        if map.is_empty() {
            return paths;
        }

        // Helper function to simply take an arbitrary point out of the map.
        // This is used to select a point to start a new closed path.
        let take_point = |map: &mut HashMap<OrderedVec2, OrderedVec2>| {
            if let Some((&k, &v)) = map.iter().next() {
                map.remove(&k);
                Some((k, v))
            } else {
                None
            }
        };

        let mut current_path = ClosedPath::new();
        let (mut segment_start, mut segment_end) = take_point(&mut map).unwrap();

        // The vector from p0 -> p1.
        let mut segment_direction = segment_end - segment_start;
        let mut path_start = segment_start;
        current_path.add_point(segment_start.x.0, segment_start.y.0);
        let mut x_limits = (0., 0.);
        let mut y_limits = (0., 0.);
        loop {
            let next = map.remove(&segment_end).unwrap();

            // Within this loop we are considering 3 points:
            //    > p0 - The start point of the current line segment. This is already added to
            //           the path.
            //    > p1 - The last known end of the current line segment.
            //    > p2 - The next possible point in the path.
            let p0 = segment_start;
            let p1 = segment_end;
            let p2 = next;

            // And we consider to direction vectors:
            //    > vp0_p1 - The direction vector from p0 -> p1
            //    > vp0_p2 - The direction vector from p0 -> p2
            let vp0_p1 = segment_direction;
            let vp0_p2 = p2 - p0;

            // We check if points p0, p1, p2 all lie on a line. If they are all colinear then
            // we can delete point p1 since it will be represented by the line p0->p2. If they
            // are not parallel then we add p1 to our path and update our segment to now be
            // p1 -> p2.
            if !is_parallel(vp0_p1, vp0_p2) {
                current_path.add_point(p1.x.0, p1.y.0);
                // Track the contour limits while assembling.
                if p1.x.0 < x_limits.0 {
                    x_limits.0 = p1.x.0
                }
                if p1.x.0 > x_limits.1 {
                    x_limits.1 = p1.x.0
                }
                if p1.y.0 < y_limits.0 {
                    y_limits.0 = p1.y.0
                }
                if p1.y.0 > y_limits.1 {
                    y_limits.1 = p1.y.0
                }
                segment_start = p1;
                segment_direction = p2 - p1;
            }
            segment_end = p2;

            // If this point is back to the start then we can close this path and create a new one.
            if next == path_start {
                // The contour start point may be in the middle of a line segment. We want to detect this situation
                // and update our start point to the start of the line segment.
                //
                // For example, in the following situation:
                //   *--------*-----------* < p1
                //   ^        ^           |
                //   p3       p2          * < p0
                //
                // Here, p2 is our path start, but that point can be removed if we instead update the path start to
                // be p1 instead.
                let vp1_p2 = p2 - p1;
                let vp3_p2 = current_path.points()[1] - current_path.points()[0];
                if is_parallel(vp3_p2, vp1_p2) {
                    let last = current_path.points().len() - 1;
                    current_path.points_vec().remove(last);
                    current_path.points_vec()[0] = Vector2 {
                        x: segment_start.x.0,
                        y: segment_start.y.0,
                    };
                }

                // Finish the current path and initialize a new one.
                paths.add_path(current_path);
                if let Some((start, end)) = take_point(&mut map) {
                    segment_start = start;
                    segment_end = end;
                    segment_direction = end - start;
                    path_start = segment_start;
                    current_path = ClosedPath::new();
                    current_path.add_point(segment_start.x.0, segment_start.y.0);
                    continue;
                } else {
                    break;
                }
            }
        }
        paths.limits_x = x_limits;
        paths.limits_y = y_limits;
        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordered_float::OrderedFloat;

    #[test]
    fn build_contour() {
        let p0 = Vector2 {
            x: OrderedFloat(0.0),
            y: OrderedFloat(0.0),
        };
        let p1 = Vector2 {
            x: OrderedFloat(1.0),
            y: OrderedFloat(0.0),
        };
        let p2 = Vector2 {
            x: OrderedFloat(1.0),
            y: OrderedFloat(1.0),
        };
        let p3 = Vector2 {
            x: OrderedFloat(0.0),
            y: OrderedFloat(1.0),
        };
        let mut map = SegmentMap::new();
        map.insert(p0, p1);
        map.insert(p1, p2);
        map.insert(p2, p3);
        map.insert(p3, p0);

        let contour = Contour::from_segment_map(map);

        assert_eq!(contour.paths().len(), 1);

        let segments = contour.paths[0].segments().collect::<Vec<_>>();
        assert_eq!(segments.len(), 4);
    }

    #[test]
    fn segment_iterator() {
        let mut path = ClosedPath::new();
        let p0 = (0.0, 0.0);
        let p1 = (1.0, 0.0);
        let p2 = (1.0, 1.0);
        let p3 = (0.0, 1.0);
        path.add_point(p0.0, p0.1);
        path.add_point(p1.0, p1.1);
        path.add_point(p2.0, p2.1);
        path.add_point(p3.0, p3.1);

        let mut segments = path.segments();
        assert_eq!(Some((p0, p1)), segments.next());
        assert_eq!(Some((p1, p2)), segments.next());
        assert_eq!(Some((p2, p3)), segments.next());
        assert_eq!(Some((p3, p0)), segments.next());
        assert_eq!(None, segments.next());
    }
}
