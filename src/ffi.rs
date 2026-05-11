use crate::{
    ColorModel, Plugin,
    param::{Color, ParamInfo, ParamKind, Position},
};
pub use frei0r_sys2::*;
use std::ffi::{CStr, c_int, c_uint};

mod private {
    pub trait Sealed<const S: usize> {}

    impl<T> Sealed<0> for T where T: crate::Plugin<0> {}
    impl<T> Sealed<1> for T where T: crate::Plugin<1> {}
    impl<T> Sealed<2> for T where T: crate::Plugin<2> {}
    impl<T> Sealed<3> for T where T: crate::Plugin<3> {}
}

pub trait PluginKind<const S: usize>: Plugin<S> + private::Sealed<S> {
    const SIZE: usize;

    fn plugin_type() -> i32;

    fn update_raw(
        &mut self,
        frame_length: usize,
        time: f64,
        inframe1: *const u32,
        inframe2: *const u32,
        inframe3: *const u32,
        outframe: &mut [u32],
    );
}

impl<T> PluginKind<0> for T
where
    T: Plugin<0>,
{
    const SIZE: usize = 0;

    fn plugin_type() -> i32 {
        F0R_PLUGIN_TYPE_SOURCE as i32
    }

    fn update_raw(
        &mut self,
        _frame_length: usize,
        time: f64,
        _inframe1: *const u32,
        _inframe2: *const u32,
        _inframe3: *const u32,
        outframe: &mut [u32],
    ) {
        self.update(time, [], outframe);
    }
}

impl<T> PluginKind<1> for T
where
    T: Plugin<1>,
{
    const SIZE: usize = 1;

    fn plugin_type() -> i32 {
        F0R_PLUGIN_TYPE_FILTER as i32
    }

    fn update_raw(
        &mut self,
        frame_length: usize,
        time: f64,
        inframe1: *const u32,
        _inframe2: *const u32,
        _inframe3: *const u32,
        outframe: &mut [u32],
    ) {
        assert!(!inframe1.is_null());
        self.update(time, [frame_to_slice(&inframe1, frame_length)], outframe);
    }
}

impl<T> PluginKind<2> for T
where
    T: Plugin<2>,
{
    const SIZE: usize = 2;

    fn plugin_type() -> i32 {
        F0R_PLUGIN_TYPE_MIXER2 as i32
    }

    fn update_raw(
        &mut self,
        frame_length: usize,
        time: f64,
        inframe1: *const u32,
        inframe2: *const u32,
        _inframe3: *const u32,
        outframe: &mut [u32],
    ) {
        assert!(!inframe1.is_null());
        assert!(!inframe2.is_null());
        self.update(
            time,
            [
                frame_to_slice(&inframe1, frame_length),
                frame_to_slice(&inframe2, frame_length),
            ],
            outframe,
        );
    }
}

impl<T> PluginKind<3> for T
where
    T: Plugin<3>,
{
    const SIZE: usize = 3;

    fn plugin_type() -> i32 {
        F0R_PLUGIN_TYPE_MIXER2 as i32
    }

    fn update_raw(
        &mut self,
        frame_length: usize,
        time: f64,
        inframe1: *const u32,
        inframe2: *const u32,
        inframe3: *const u32,
        outframe: &mut [u32],
    ) {
        assert!(!inframe1.is_null());
        assert!(!inframe2.is_null());
        assert!(!inframe3.is_null());
        self.update(
            time,
            [
                frame_to_slice(&inframe1, frame_length),
                frame_to_slice(&inframe2, frame_length),
                frame_to_slice(&inframe3, frame_length),
            ],
            outframe,
        );
    }
}

pub struct Instance<P, const S: usize>
where
    P: Plugin<S> + PluginKind<S>,
{
    frame_length: usize,
    inner: P,
}

impl<P, const S: usize> Instance<P, S>
where
    P: Plugin<S> + PluginKind<S>,
{
    /// # Safety
    /// frei0r contract
    pub unsafe fn f0r_get_plugin_info(info: *mut f0r_plugin_info_t) {
        let info = unsafe { &mut *info };
        let our_info = P::info();

        info.name = our_info.name.as_ptr();
        info.author = our_info.author.as_ptr();
        info.plugin_type = P::plugin_type();
        info.color_model = match our_info.color_model {
            ColorModel::BGRA8888 => F0R_COLOR_MODEL_BGRA8888 as i32,
            ColorModel::RGBA8888 => F0R_COLOR_MODEL_RGBA8888 as i32,
            ColorModel::PACKED32 => F0R_COLOR_MODEL_PACKED32 as i32,
        };
        info.frei0r_version = FREI0R_MAJOR_VERSION as i32;
        info.major_version = our_info.major_version;
        info.minor_version = our_info.minor_version;
        info.num_params = P::PARAMS.len() as i32;
        if let Some(explanation) = our_info.explanation {
            info.explanation = explanation.as_ptr();
        }
    }

    /// # Safety
    /// frei0r contract
    pub unsafe fn f0r_get_param_info(info: *mut f0r_param_info_t, param_index: c_int) {
        let param_index = param_index as usize;

        let info = unsafe { &mut *info };
        let our_info: &ParamInfo<P> = &P::PARAMS[param_index];

        info.name = our_info.name().as_ptr();
        info.type_ = match our_info.kind() {
            ParamKind::Bool { .. } => F0R_PARAM_BOOL as i32,
            ParamKind::Double { .. } => F0R_PARAM_DOUBLE as i32,
            ParamKind::Color { .. } => F0R_PARAM_COLOR as i32,
            ParamKind::Position { .. } => F0R_PARAM_POSITION as i32,
            ParamKind::String { .. } => F0R_PARAM_STRING as i32,
        };
        info.explanation = our_info.explanation().as_ptr();
    }

    pub fn new(width: c_uint, height: c_uint) -> Self {
        let width = width.try_into().unwrap();
        let height = height.try_into().unwrap();
        let plugin = P::new(width, height);
        Self {
            frame_length: width * height,
            inner: plugin,
        }
    }

    pub fn f0r_set_param_value(&mut self, param: f0r_param_t, param_index: c_int) {
        let param_index = param_index as usize;
        let param_info: &ParamInfo<P> = &P::PARAMS[param_index];
        let kind = param_info.kind();
        match kind {
            ParamKind::Bool { set, .. } => {
                let param = unsafe { *(param as *const f0r_param_bool) };
                set(&mut self.inner, param >= 0.5);
            }
            ParamKind::Double { set, .. } => {
                let param = unsafe { *(param as *const f0r_param_double) };
                set(&mut self.inner, param);
            }
            ParamKind::Color { set, .. } => {
                let param = unsafe { *(param as *const f0r_param_color) };
                let color = Color {
                    r: param.r,
                    g: param.g,
                    b: param.b,
                };
                set(&mut self.inner, &color);
            }
            ParamKind::Position { set, .. } => {
                let param = unsafe { *(param as *const f0r_param_position) };
                let position = Position {
                    x: param.x,
                    y: param.y,
                };
                set(&mut self.inner, &position);
            }
            ParamKind::String { set, .. } => {
                let param = unsafe { *(param as *const f0r_param_string) };
                let string = unsafe { CStr::from_ptr(param) };
                set(&mut self.inner, string);
            }
        };
    }

    pub fn f0r_get_param_value(&self, param: f0r_param_t, param_index: c_int) {
        let param_index = param_index as usize;
        let param_info: &ParamInfo<P> = &P::PARAMS[param_index];
        let kind = param_info.kind();
        match kind {
            ParamKind::Bool { get, .. } => {
                let param = unsafe { &mut *(param as *mut f0r_param_bool) };
                *param = if get(&self.inner) { 1.0 } else { 0.0 };
            }
            ParamKind::Double { get, .. } => {
                let param = unsafe { &mut *(param as *mut f0r_param_double) };
                *param = get(&self.inner);
            }
            ParamKind::Color { get, .. } => {
                let param = unsafe { &mut *(param as *mut f0r_param_color) };
                let color = get(&self.inner);
                param.r = color.r;
                param.g = color.g;
                param.b = color.b;
            }
            ParamKind::Position { get, .. } => {
                let param = unsafe { &mut *(param as *mut f0r_param_position) };
                let position = get(&self.inner);
                param.x = position.x;
                param.y = position.y;
            }
            ParamKind::String { get, .. } => {
                let param = unsafe { &mut *(param as *mut f0r_param_string) };
                // We are casting away constness here. This should be fine since quoting the
                // comment found in the original header, "If the caller needs to modify the
                // value, it should make a copy of it and modify before calling
                // f0r_set_param_value()."
                *param = get(&self.inner).as_ptr() as f0r_param_string;
            }
        };
    }

    /// # Safety
    /// frei0r contract
    pub unsafe fn f0r_update2(
        &mut self,
        time: f64,
        inframe1: *const u32,
        inframe2: *const u32,
        inframe3: *const u32,
        outframe: *mut u32,
    ) {
        assert!(!outframe.is_null());
        let outframe = unsafe { std::slice::from_raw_parts_mut(outframe, self.frame_length) };
        self.inner.update_raw(
            self.frame_length,
            time,
            inframe1,
            inframe2,
            inframe3,
            outframe,
        );
    }
}

fn frame_to_slice(frame: &*const u32, length: usize) -> &[u32] {
    if frame.is_null() {
        panic!("Unexpected null frame");
    } else {
        unsafe { std::slice::from_raw_parts(*frame, length) }
    }
}
