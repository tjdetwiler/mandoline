pub type Vector3 = cgmath::Vector3<f32>;

// We rely on Vector3 being repr(c).
static_assertions::assert_eq_size!(Vector3, [f32; 3]);
static_assertions::assert_eq_align!(Vector3, f32);

pub struct Triangle {
    pub p0: Vector3,
    pub p1: Vector3,
    pub p2: Vector3,
}
