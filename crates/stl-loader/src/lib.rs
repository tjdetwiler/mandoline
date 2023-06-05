use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek};
use std::path::Path;
use zerocopy::AsBytes;

pub struct StlFile {
    triangles: usize,
    data: Vec<f32>,
}

impl StlFile {
    /// Returns the number of triangles included in the STL file.
    ///
    /// Each triangle is represented as 9 32-bit floating point numbers.
    pub fn triangles(&self) -> usize {
        self.triangles
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_bytes()
    }

    pub fn as_floats(&self) -> &[f32] {
        &self.data
    }

    pub fn into_inner(self) -> Vec<f32> {
        let StlFile { data, .. } = self;
        data
    }
}

fn read_binary<T: Read + Seek>(f: &mut T) -> std::io::Result<StlFile> {
    // Binary files start with an 80 byte header. There is no defined structure for this
    // header but some implementations will stash some metadata in this header. For now
    // we'll just skip the header and load the geometry.
    f.seek(std::io::SeekFrom::Start(80))?;

    // Immediately following the header is an unsigned 32-bit integer that indicates the
    // number of triagles that follow.
    let n_triangles = f.read_u32::<LittleEndian>()? as usize;

    // We have 9 floats for each triangle; x,y,z for each of the 3 vertices.
    let mut data = Vec::<f32>::with_capacity(n_triangles * 3 * 3);
    let mut slice_buf: [f32; 9] = Default::default();
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
        for float in &mut slice_buf {
            *float = f.read_f32::<LittleEndian>()?;
        }
        data.extend_from_slice(&slice_buf);
        // After the triangle geometry there is a 2-byte unsigned integer called the
        // "attribute byte count". There is no standard structure of this field, but
        // some applications use this for color data.
        let _attribute_byte_count = f.read_u16::<LittleEndian>()?;
    }
    Ok(StlFile {
        triangles: n_triangles,
        data,
    })
}

pub fn read_stl<P: AsRef<Path>>(p: P) -> std::io::Result<StlFile> {
    let mut f = std::fs::File::open(p)?;
    read_binary(&mut f)
}

pub fn parse_stl(data: &[u8]) -> std::io::Result<StlFile> {
    let mut c = std::io::Cursor::new(data);
    read_binary(&mut c)
}

pub trait StlReader: Read {
    fn read_stl(&mut self) -> std::io::Result<StlFile>;
}

impl<T: Read + Seek> StlReader for T {
    fn read_stl(&mut self) -> std::io::Result<StlFile> {
        read_binary(self)
    }
}
