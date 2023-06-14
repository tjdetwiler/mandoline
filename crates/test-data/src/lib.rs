pub struct TestModel {
    pub bytes: &'static [u8],
    pub model_height: f32,
}

pub const STL_CUBE: TestModel = TestModel {
    bytes: include_bytes!("../../../res/cube/cube-bin.stl"),
    model_height: 20.0,
};

pub const STL_CALIBRATION_CUBE: TestModel = TestModel {
    bytes: include_bytes!("../../../res/calibration-cube/cube-bin.stl"),
    model_height: 20.0,
};
