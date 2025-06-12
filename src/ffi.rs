use crate::param::{Color, ParamInfo, ParamKind, Position};
use crate::{ColorModel, FilterPlugin, Mixer2Plugin, Mixer3Plugin, Plugin, SourcePlugin};
pub use frei0r_sys::*;
use std::ffi::{CStr, c_int, c_uint};

mod private {
    pub trait Sealed {}
}

/// Marker trait to determine type of plugin.
/// See [KindSource], [KindFilter], [KindMixer2], [KindMixer3]
pub trait PluginKind: private::Sealed {
    #[doc(hidden)]
    const PLUGIN_TYPE: i32;
}

/// Marker type representing a Source plugin.
#[derive(Debug, Clone, Copy)]
pub struct KindSource;
impl private::Sealed for KindSource {}
impl PluginKind for KindSource {
    #[doc(hidden)]
    const PLUGIN_TYPE: i32 = F0R_PLUGIN_TYPE_SOURCE as i32;
}

/// Marker type representing a Filter plugin.
#[derive(Debug, Clone, Copy)]
pub struct KindFilter;
impl private::Sealed for KindFilter {}
impl PluginKind for KindFilter {
    #[doc(hidden)]
    const PLUGIN_TYPE: i32 = F0R_PLUGIN_TYPE_FILTER as i32;
}

/// Marker type representing a Mixer2 plugin.
#[derive(Debug, Clone, Copy)]
pub struct KindMixer2;
impl private::Sealed for KindMixer2 {}
impl PluginKind for KindMixer2 {
    #[doc(hidden)]
    const PLUGIN_TYPE: i32 = F0R_PLUGIN_TYPE_MIXER2 as i32;
}

/// Marker type representing a Mixer3 plugin.
#[derive(Debug, Clone, Copy)]
pub struct KindMixer3;
impl private::Sealed for KindMixer3 {}
impl PluginKind for KindMixer3 {
    #[doc(hidden)]
    const PLUGIN_TYPE: i32 = F0R_PLUGIN_TYPE_MIXER3 as i32;
}

// Bridges between type-level plugin kinds and runtime update behavior.
//
// This trait is parameterized by PluginKind `K` to avoid implementation conflicts
// while providing a uniform `update` signature for the FFI layer. Each plugin type
// (Source, Filter, Mixer2, Mixer3) has different update method signatures,
// this trait dispatches to the appropriate plugin-specific method based on
// the PluginKind type parameter `K`.
#[doc(hidden)]
pub trait PluginKindUpdate<K: PluginKind> {
    fn update(
        &mut self,
        frame_length: usize,
        time: f64,
        inframe1: *const u32,
        inframe2: *const u32,
        inframe3: *const u32,
        outframe: &mut [u32],
    );
}

impl<T> PluginKindUpdate<KindSource> for T
where
    T: SourcePlugin,
{
    fn update(
        &mut self,
        _frame_length: usize,
        time: f64,
        _inframe1: *const u32,
        _inframe2: *const u32,
        _inframe3: *const u32,
        outframe: &mut [u32],
    ) {
        self.update_source(time, outframe);
    }
}

impl<T> PluginKindUpdate<KindFilter> for T
where
    T: FilterPlugin,
{
    fn update(
        &mut self,
        frame_length: usize,
        time: f64,
        inframe1: *const u32,
        _inframe2: *const u32,
        _inframe3: *const u32,
        outframe: &mut [u32],
    ) {
        self.update_filter(time, frame_to_slice(&inframe1, frame_length), outframe);
    }
}

impl<T> PluginKindUpdate<KindMixer2> for T
where
    T: Mixer2Plugin,
{
    fn update(
        &mut self,
        frame_length: usize,
        time: f64,
        inframe1: *const u32,
        inframe2: *const u32,
        _inframe3: *const u32,
        outframe: &mut [u32],
    ) {
        self.update_mixer2(
            time,
            frame_to_slice(&inframe1, frame_length),
            frame_to_slice(&inframe2, frame_length),
            outframe,
        );
    }
}

impl<T> PluginKindUpdate<KindMixer3> for T
where
    T: Mixer3Plugin,
{
    fn update(
        &mut self,
        frame_length: usize,
        time: f64,
        inframe1: *const u32,
        inframe2: *const u32,
        inframe3: *const u32,
        outframe: &mut [u32],
    ) {
        self.update_mixer3(
            time,
            frame_to_slice(&inframe1, frame_length),
            frame_to_slice(&inframe2, frame_length),
            frame_to_slice(&inframe3, frame_length),
            outframe,
        );
    }
}

#[doc(hidden)]
pub struct Instance<P: Plugin + PluginKindUpdate<P::Kind>> {
    frame_length: usize,
    inner: P,
}

impl<P> Instance<P>
where
    P: Plugin + PluginKindUpdate<P::Kind>,
{
    pub unsafe fn f0r_get_plugin_info(info: *mut f0r_plugin_info_t) {
        let info = unsafe { &mut *info };
        let our_info = P::info();

        info.name = our_info.name.as_ptr();
        info.author = our_info.author.as_ptr();
        info.plugin_type = P::Kind::PLUGIN_TYPE;
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

    pub unsafe fn f0r_update2(
        &mut self,
        time: f64,
        inframe1: *const u32,
        inframe2: *const u32,
        inframe3: *const u32,
        outframe: *mut u32,
    ) {
        if outframe.is_null() {
            panic!("unexpected null output frame");
        }
        let outframe = unsafe { std::slice::from_raw_parts_mut(outframe, self.frame_length) };
        <P as PluginKindUpdate<P::Kind>>::update(
            &mut self.inner,
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
