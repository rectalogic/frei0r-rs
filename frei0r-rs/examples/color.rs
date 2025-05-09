use frei0r_rs::*;

#[derive(PluginBase)]
pub struct ColorPlugin {
    #[frei0r(explain = c"Color to generate")]
    color: Color,
}

impl Plugin for ColorPlugin {
    fn info() -> PluginInfo {
        PluginInfo {
            name: c"frei0r-rs color",
            author: c"none",
            plugin_type: PluginType::Source,
            color_model: ColorModel::RGBA8888,
            major_version: 1,
            minor_version: 0,
            explanation: c"Source plugin used for the testing of frei0r-rs",
        }
    }

    fn new(_width: usize, _height: usize) -> Self {
        Self {
            color: Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
        }
    }

    fn source_update(&mut self, _time: f64, width: usize, height: usize, outframe: &mut [u32]) {
        let r_u8 = (self.color.r * 255.0) as u8;
        let g_u8 = (self.color.g * 255.0) as u8;
        let b_u8 = (self.color.b * 255.0) as u8;
        let a_u8 = 255;
        let pixel =
            ((r_u8 as u32) << 24) | ((g_u8 as u32) << 16) | ((b_u8 as u32) << 8) | (a_u8 as u32);

        for dy in 0..height {
            for dx in 0..width {
                outframe[dy * width + dx] = pixel;
            }
        }
    }
}

plugin!(ColorPlugin);
