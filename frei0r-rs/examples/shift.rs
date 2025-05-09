use frei0r_rs::*;

struct Extra;

#[derive(PluginBase)]
pub struct ShiftPlugin {
    #[frei0r(explain = c"Shift in x direction")]
    xshift: f64,
    #[frei0r(explain = c"Shift in y direction")]
    yshift: f64,
    // Private internal field
    extra: Extra,
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

    fn new(_width: usize, _height: usize) -> Self {
        Self {
            xshift: 0.0,
            yshift: 0.0,
            extra: Extra,
        }
    }

    fn filter_update(
        &mut self,
        _time: f64,
        width: usize,
        height: usize,
        inframe: &[u32],
        outframe: &mut [u32],
    ) {
        let xshift = (self.xshift * width as f64) as usize;
        let yshift = (self.yshift * height as f64) as usize;
        for dy in 0..height {
            for dx in 0..width {
                let sy = (dy + yshift) % height;
                let sx = (dx + xshift) % width;
                outframe[dy * width + dx] = inframe[sy * width + sx];
            }
        }
        // Do something with internal field
        let _extra = &self.extra;
    }
}

plugin!(ShiftPlugin);
