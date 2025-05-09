use frei0r_rs::*;

#[derive(PluginBase)]
pub struct ShiftPlugin {
    #[frei0r(explain = c"Shift in x direction")]
    xshift: f64,
    #[frei0r(explain = c"Shift in y direction")]
    yshift: f64,
    width: usize,
    height: usize,
}

impl Plugin for ShiftPlugin {
    fn info() -> PluginInfo {
        PluginInfo {
            name: c"frei0r-rs shift",
            author: c"none",
            plugin_type: PluginType::Filter,
            color_model: ColorModel::RGBA8888,
            major_version: 1,
            minor_version: 0,
            explanation: c"Filter plugin used for the testing of frei0r-rs",
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

    fn filter_update(&mut self, _time: f64, inframe: &[u32], outframe: &mut [u32]) {
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
