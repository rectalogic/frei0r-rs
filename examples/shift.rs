use frei0r_rs2::*;

pub struct ShiftPlugin {
    xshift: f64,
    yshift: f64,
    width: usize,
    height: usize,
}

impl Plugin for ShiftPlugin {
    type Kind = KindFilter;

    const PARAMS: &'static [param::ParamInfo<Self>] = &[
        param::ParamInfo::new_double(
            c"xshift",
            c"Shift in x direction",
            |plugin| plugin.xshift,
            |plugin, value| plugin.xshift = value,
        ),
        param::ParamInfo::new_double(
            c"yshift",
            c"Shift in y direction",
            |plugin| plugin.yshift,
            |plugin, value| plugin.yshift = value,
        ),
    ];

    fn info() -> PluginInfo {
        PluginInfo {
            name: c"frei0r-rs2 shift",
            author: c"none",
            color_model: ColorModel::RGBA8888,
            major_version: 1,
            minor_version: 0,
            explanation: Some(c"Filter plugin used for the testing of frei0r-rs2"),
        }
    }

    fn new(width: usize, height: usize) -> Self {
        Self {
            xshift: 0.0,
            yshift: 0.0,
            width,
            height,
        }
    }
}

impl FilterPlugin for ShiftPlugin {
    fn update_filter(&mut self, _time: f64, inframe: &[u32], outframe: &mut [u32]) {
        let xshift = (self.xshift * self.width as f64) as usize;
        let yshift = (self.yshift * self.height as f64) as usize;
        for dy in 0..self.height {
            for dx in 0..self.width {
                let sy = (dy + yshift) % self.height;
                let sx = (dx + xshift) % self.width;
                outframe[dy * self.width + dx] = inframe[sy * self.width + sx];
            }
        }
    }
}

plugin!(ShiftPlugin);
