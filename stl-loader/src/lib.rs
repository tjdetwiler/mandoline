use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::Seek;
use std::path::Path;
use zerocopy::AsBytes;

#[derive(AsBytes, Copy, Clone, Debug, PartialEq)]
#[repr(packed)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(AsBytes, Copy, Clone, Debug, PartialEq)]
#[repr(packed)]
pub struct Triangle {
    pub normal: Vertex,
    pub vertices: [Vertex; 3],
    pub attribute_byte_count: u16,
}

pub struct StlFile {
    pub triangles: Vec<Triangle>,
}

fn read_vertex(f: &mut File) -> std::io::Result<Vertex> {
    Ok(Vertex {
        x: f.read_f32::<LittleEndian>()?,
        y: f.read_f32::<LittleEndian>()?,
        z: f.read_f32::<LittleEndian>()?,
    })
}

fn read_binary(f: &mut File) -> Result<StlFile, anyhow::Error> {
    // Binary files start with an 80 byte header. There is no defined structure for this
    // header but some implementations will stash some metadata in this header. For now
    // we'll just skip the header and load the geometry.
    f.seek(std::io::SeekFrom::Start(80))?;

    // Immediately following the header is an unsigned 32-bit integer that indicates the
    // number of triagles that follow.
    let n = f.read_u32::<LittleEndian>()?;

    let mut triangles = Vec::<Triangle>::with_capacity(n as usize);
    for _ in 0..n {
        // Each triangle is specified by a normal vector followed by 3 verticies of the
        // triangle. While the normal vector may be included, it is generally expected
        // that verticies be listed in counter-clockwise order and so the normal vector
        // maybe specified as (0, 0, 0).
        triangles.push(Triangle {
            normal: read_vertex(f)?,
            vertices: [read_vertex(f)?, read_vertex(f)?, read_vertex(f)?],
            // After the triangle geometry there is a 2-byte unsigned integer called the
            // "attribute byte count". There is no standard structure of this field, but
            // some applications use this for color data.
            attribute_byte_count: f.read_u16::<LittleEndian>()?,
        });
    }
    Ok(StlFile { triangles })
}

pub fn read_stl<P: AsRef<Path>>(p: P) -> Result<StlFile, anyhow::Error> {
    let mut f = std::fs::File::open(p)?;
    read_binary(&mut f)
}
