use byteorder::{LittleEndian, ReadBytesExt};
use mandoline_mesh::{Triangle, TriangleMesh, Vector3};
use std::io::{Read, Seek};
use std::path::Path;

fn read_binary<M: TriangleMesh, T: Read + Seek>(f: &mut T) -> std::io::Result<M> {
    // Binary files start with an 80 byte header. There is no defined structure for this
    // header but some implementations will stash some metadata in this header. For now
    // we'll just skip the header and load the geometry.
    f.seek(std::io::SeekFrom::Start(80))?;

    // Immediately following the header is an unsigned 32-bit integer that indicates the
    // number of triagles that follow.
    let n_triangles = f.read_u32::<LittleEndian>()? as usize;

    // We have 9 floats for each triangle; x,y,z for each of the 3 vertices.
    let mut data = Vec::<Triangle>::with_capacity(n_triangles);
    for _ in 0..n_triangles {
        // Each triangle is specified by a normal vector followed by 3 verticies of the
        // triangle. While the normal vector may be included, it is generally expected
        // that verticies be listed in counter-clockwise order and so the normal vector
        // maybe specified as (0, 0, 0).
        let _normal = (
            f.read_f32::<LittleEndian>()?,
            f.read_f32::<LittleEndian>()?,
            f.read_f32::<LittleEndian>()?,
        );
        data.push(Triangle {
            p0: Vector3 {
                x: f.read_f32::<LittleEndian>()?,
                y: f.read_f32::<LittleEndian>()?,
                z: f.read_f32::<LittleEndian>()?,
            },
            p1: Vector3 {
                x: f.read_f32::<LittleEndian>()?,
                y: f.read_f32::<LittleEndian>()?,
                z: f.read_f32::<LittleEndian>()?,
            },
            p2: Vector3 {
                x: f.read_f32::<LittleEndian>()?,
                y: f.read_f32::<LittleEndian>()?,
                z: f.read_f32::<LittleEndian>()?,
            },
        });
        // After the triangle geometry there is a 2-byte unsigned integer called the
        // "attribute byte count". There is no standard structure of this field, but
        // some applications use this for color data.
        let _attribute_byte_count = f.read_u16::<LittleEndian>()?;
    }
    Ok(M::from_triangles(data))
}

pub fn read_stl<M: TriangleMesh, P: AsRef<Path>>(p: P) -> std::io::Result<M> {
    let mut f = std::fs::File::open(p)?;
    read_binary(&mut f)
}

pub fn parse_stl<M: TriangleMesh>(data: &[u8]) -> std::io::Result<M> {
    let mut c = std::io::Cursor::new(data);
    read_binary(&mut c)
}

pub trait StlReader: Read {
    fn read_stl<M: TriangleMesh>(&mut self) -> std::io::Result<M>;
}

impl<T: Read + Seek> StlReader for T {
    fn read_stl<M: TriangleMesh>(&mut self) -> std::io::Result<M> {
        read_binary(self)
    }
}
