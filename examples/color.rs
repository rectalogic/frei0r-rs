use frei0r_rs2::*;

pub struct ColorPlugin {
    color: Color,
    width: usize,
    height: usize,
}

impl Plugin for ColorPlugin {
    type Kind = KindSource;

    const PARAMS: &'static [ParamInfo<Self>] = &[ParamInfo::new_color(
        c"color",
        c"Color to generate",
        |plugin| &plugin.color,
        |plugin, value| plugin.color = *value,
    )];

    fn info() -> PluginInfo {
        PluginInfo {
            name: c"frei0r-rs2 color",
            author: c"none",
            color_model: ColorModel::RGBA8888,
            major_version: 1,
            minor_version: 0,
            explanation: Some(c"Source plugin used for the testing of frei0r-rs2"),
        }
    }

    fn new(width: usize, height: usize) -> Self {
        Self {
            color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            width,
            height,
        }
    }
}

impl SourcePlugin for ColorPlugin {
    fn update_source(&mut self, _time: f64, outframe: &mut [u32]) {
        let r_u8 = (self.color.r * 255.0) as u8;
        let g_u8 = (self.color.g * 255.0) as u8;
        let b_u8 = (self.color.b * 255.0) as u8;
        let a_u8 = 255;
        let pixel =
            ((r_u8 as u32) << 24) | ((g_u8 as u32) << 16) | ((b_u8 as u32) << 8) | (a_u8 as u32);

        for dy in 0..self.height {
            for dx in 0..self.width {
                outframe[dy * self.width + dx] = pixel;
            }
        }
    }
}

plugin!(ColorPlugin);
