use frei0r_rs::*;

pub struct TestPlugin {
    xshift : f64,
    yshift : f64,
}

unsafe impl PluginBase for TestPlugin {
    fn param_count() -> usize {
        2
    }

    fn param_info(param_index : usize) -> ParamInfo {
        match param_index {
            0 => ParamInfo { name : c"xshift", param_type : ParamType::Double, explanation : c"shift in x direction" },
            1 => ParamInfo { name : c"yshift", param_type : ParamType::Double, explanation : c"shift in y direction" },
            _ => unreachable!(),
        }
    }

    fn param(&self, param_index : usize) -> Param<'_> {
        match param_index {
            0 => Param::Double(&self.xshift),
            1 => Param::Double(&self.yshift),
            _ => unreachable!(),
        }
    }

    fn param_mut(&mut self, param_index : usize) -> ParamMut<'_> {
        match param_index {
            0 => ParamMut::Double(&mut self.xshift),
            1 => ParamMut::Double(&mut self.yshift),
            _ => unreachable!(),
        }
    }
}

impl Plugin for TestPlugin {
    fn info() -> PluginInfo {
        PluginInfo {
            name : c"Test",
            author : c"none",
            plugin_type : PluginType::Filter,
            color_model : ColorModel::RGBA8888,
            major_version : 1,
            minor_version : 0,
            explanation : c"Plugin used for the testing of frei0r-rs",
        }
    }

    fn new(_width : usize, _height : usize) -> Self {
        Self {
            xshift : 0.0,
            yshift : 0.0,
        }
    }

    fn update(&self, _time : f64, width : usize, height : usize, inframe : &[u32], outframe : &mut [u32]) {
        let xshift = (self.xshift * width  as f64) as usize;
        let yshift = (self.yshift * height as f64) as usize;
        for dy in 0..height {
            for dx in 0..width {
                let sy = (dy + yshift) % height;
                let sx = (dx + xshift) % width;
                outframe[dy * width + dx] = inframe[sy * width + sx];
            }
        }
    }

    fn update2(&self, _ : f64, _width : usize, _height : usize, _inframe1 : &[u32], _inframe2 : &[u32], _inframe3 : &[u32], _outframe : &mut [u32]) {
        unreachable!()
    }
}

plugin!(TestPlugin);
