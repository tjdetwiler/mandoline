use mandoline_mesh::{Triangle, TriangleMesh, Vector3, VertexIndex};

const STL_CUBE: &[u8] = include_bytes!("../../../res/cube/cube-bin.stl");

#[test]
fn create_cube_mesh() {
    let mesh = mandoline_stl::parse_stl::<VertexIndex>(STL_CUBE).unwrap();

    // Expect a cube from 0-20 on x,y,z.
    let x0 = Vector3 {
        x: 0.0,
        y: 20.0,
        z: 20.0,
    };
    let x1 = Vector3 {
        x: 20.0,
        y: 0.0,
        z: 20.0,
    };
    let x2 = Vector3 {
        x: 20.0,
        y: 20.0,
        z: 20.0,
    };
    let x3 = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 20.0,
    };
    let x4 = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let x5 = Vector3 {
        x: 20.0,
        y: 20.0,
        z: 0.0,
    };
    let x6 = Vector3 {
        x: 20.0,
        y: 0.0,
        z: 0.0,
    };
    let x7 = Vector3 {
        x: 0.0,
        y: 20.0,
        z: 0.0,
    };

    let mut triangles = mesh.triangles();
    assert_eq!(
        Some(Triangle {
            p0: x0,
            p1: x1,
            p2: x2
        }),
        triangles.next()
    );
    assert_eq!(
        Some(Triangle {
            p0: x1,
            p1: x0,
            p2: x3
        }),
        triangles.next()
    );
    assert_eq!(
        Some(Triangle {
            p0: x4,
            p1: x5,
            p2: x6
        }),
        triangles.next()
    );
    assert_eq!(
        Some(Triangle {
            p0: x5,
            p1: x4,
            p2: x7
        }),
        triangles.next()
    );
    assert_eq!(
        Some(Triangle {
            p0: x4,
            p1: x1,
            p2: x3
        }),
        triangles.next()
    );
    assert_eq!(
        Some(Triangle {
            p0: x1,
            p1: x4,
            p2: x6
        }),
        triangles.next()
    );
    assert_eq!(
        Some(Triangle {
            p0: x1,
            p1: x5,
            p2: x2
        }),
        triangles.next()
    );
    assert_eq!(
        Some(Triangle {
            p0: x5,
            p1: x1,
            p2: x6
        }),
        triangles.next()
    );
    assert_eq!(
        Some(Triangle {
            p0: x5,
            p1: x0,
            p2: x2
        }),
        triangles.next()
    );
    assert_eq!(
        Some(Triangle {
            p0: x0,
            p1: x5,
            p2: x7
        }),
        triangles.next()
    );
    assert_eq!(
        Some(Triangle {
            p0: x4,
            p1: x0,
            p2: x7
        }),
        triangles.next()
    );
    assert_eq!(
        Some(Triangle {
            p0: x0,
            p1: x4,
            p2: x3
        }),
        triangles.next()
    );
    assert_eq!(None, triangles.next());
}
