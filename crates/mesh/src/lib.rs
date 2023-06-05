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
    /// * `points` - a vector which will be interpreted as a sequence of 3 dimensional vectors
    /// (x, y, z), with 3 discrete vectors per triangle. This means each triangle will be encoded
    /// using 9 32-bit floating point values. The `points` vector _must_ have a length that is a
    /// multiple of 9.
    ///
    /// # Returns
    ///
    /// `None` if `points` does not have a length that is aligned to a multiple of 9.
    fn from_triangles(points: Vec<f32>) -> Option<Self>;

    /// Returns the number of triangles that comprises this mesh.
    fn triangle_count(&self) -> usize;

    /// Returns a slice that represents a series of triangles.
    ///
    /// As this is returning a slice, this will only return `Some` if the
    /// implementation already stores the mesh in this format.
    fn as_triangle_slice(&self) -> Option<&[Triangle]>;
}
