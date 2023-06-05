use crate::geometry::Vector3;
use crate::{Triangle, TriangleMesh};

pub struct Facet {
    pub p0: u32,
    pub p1: u32,
    pub p2: u32,
}

/// Stores a triangle mesh using an array of vectors, and an array of facets
///
/// Facets only store the indices into the vector array. We do this so we can store each
/// vertex as 4 bytes instead of the 12 bytes required to store the entire Vector3. This
/// has further savings if a vertex is reused.
///
/// As a simple example, consider a simple geometry such as:
///
///    *-------*
///    |\     /|
///    | \   / |
///    |  \ /  |
///    |   *   |
///    |  / \  |
///    | /   \ |
///    |/     \|
///    *-------*
///
/// Here we have 5 points and 4 facets. If we would store every facet as a series of points
/// we would need:
///    3 floats * 4b * 3 points * 4 facets = 144 bytes.
///
/// If instead we store:
///    3 floats * 4b * 5 points  = 60b
///  + 3 indices * 4b * 4 facets = 48b
///                              =======
///                               108b
pub struct VertexIndex {
    pub points: Vec<Vector3>,
    pub facets: Vec<Facet>,
}

impl TriangleMesh for VertexIndex {
    fn from_triangles(points: Vec<f32>) -> Option<Self> {
        if points.len() % 9 != 0 {
            return None;
        }

        // Safety: we have a raw vector of f32 floating point numbers, where each triangle is
        // represented as 3 consecutive floats. We want to convert this to a vector of Vector3
        // which has the same layout as a vector of f32, just with a 12 byte stride instead of
        // 4b (and of course 1/3 of the length).
        //
        // We use manually drop to prevent rust from deleting the vector storage. This is safe
        // because we pass the pointer to a new vector that will handle deleting on drop.
        let points: Vec<Vector3> = unsafe {
            let mut points = std::mem::ManuallyDrop::new(points);
            let len = points.len();
            let ptr = points.as_mut_ptr();
            Vec::from_raw_parts(std::mem::transmute(ptr), len / 3, len / 3)
        };
        Some(VertexIndex {
            // STL files provide one point for every facet vertex, so this is simply an identity
            // mapping (ex: facet[i] == i).
            //
            // As a future optimization we should de-duplicate the points vector.
            facets: (0..points.len() as u32 / 3)
                .map(|i| Facet {
                    p0: 3 * i,
                    p1: 3 * i + 1,
                    p2: 3 * i + 2,
                })
                .collect(),
            points,
        })
    }

    fn triangle_count(&self) -> usize {
        self.facets.len()
    }

    fn as_triangle_slice(&self) -> Option<&[Triangle]> {
        let points = self.points.as_slice();
        unsafe {
            let (prefix, triangles, suffix) = points.align_to::<Triangle>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            Some(triangles)
        }
    }
}
