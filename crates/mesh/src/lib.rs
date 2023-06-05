mod geometry;
mod vertex_index;

pub use geometry::*;
pub use vertex_index::*;

/// A reasonable default mesh to select for unopinionated consumers.
pub type DefaultMesh = VertexIndex;

pub trait TriangleMesh: Sized {
    /// Creates a TriangleMesh from a vector of floats that encode a list of triangles.
    ///
    /// # Arguments
    ///
    /// * `triangles` - A vector of the triangles of the mesh.
    fn from_triangles(triangles: Vec<Triangle>) -> Self;

    /// Returns the number of triangles that comprises this mesh.
    fn triangle_count(&self) -> usize;

    /// Returns a slice that represents a series of triangles.
    ///
    /// As this is returning a slice, this will only return `Some` if the
    /// implementation already stores the mesh in this format.
    fn as_triangle_slice(&self) -> Option<&[Triangle]>;
}
