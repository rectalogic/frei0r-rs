use frei0r_rs::*;

pub struct TestPlugin {
    xshift : Param,
    yshift : Param,
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
            num_params : 2,
            explanation : c"Plugin used for the testing of frei0r-rs",
        }
    }

    fn param_info(param_index : usize) -> ParamInfo {
        match param_index {
            0 => ParamInfo { name : c"xshift", param_type : ParamType::Double, explanation : c"shift in x direction" },
            1 => ParamInfo { name : c"yshift", param_type : ParamType::Double, explanation : c"shift in y direction" },
            _ => unreachable!(),
        }
    }

    fn new(_width : usize, _height : usize) -> Self {
        Self {
            xshift : Param::Double(0.0),
            yshift : Param::Double(0.0),
        }
    }

    fn param(&self, param_index : usize) -> &Param {
        match param_index {
            0 => &self.xshift,
            1 => &self.yshift,
            _ => unreachable!(),
        }
    }

    fn param_mut(&mut self, param_index : usize) -> &mut Param {
        match param_index {
            0 => &mut self.xshift,
            1 => &mut self.yshift,
            _ => unreachable!(),
        }
    }

    fn update(&self, _time : f64, width : usize, height : usize, inframe : &[u32], outframe : &mut [u32]) {
        let xshift = (match self.xshift { Param::Double(value) => value, _ => unreachable!() } * width  as f64) as usize;
        let yshift = (match self.yshift { Param::Double(value) => value, _ => unreachable!() } * height as f64) as usize;
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
