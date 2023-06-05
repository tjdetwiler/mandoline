# Mandoline Mesh

This crate implements data structures for triangle meshes that can be used
by the mandoline slicer libraries. For the time being, the goal of this
crate is to allow for different triangle mesh data structures to be
evaluated for performance characteristics. I expect that over time the data
structures may convege to a single concrete type that is ideal but it's
helpful to be able to use simpler data structures in the earlier stages of
development.

## Design

The goal of the API of this crate is to mostly have consumers depend on the
`TriangleMesh` trait. This means that `TriangleMesh` may need to be a super-set of functionality.