use crate::ColorModel;
use crate::ParamKind;
use crate::ParamMut;
use crate::ParamRef;
use crate::Plugin;
use crate::PluginType;

pub use frei0r_sys::f0r_instance_t;
pub use frei0r_sys::f0r_param_info_t;
pub use frei0r_sys::f0r_param_t;
pub use frei0r_sys::f0r_plugin_info_t;

pub use std::ffi::c_int;
pub use std::ffi::c_uint;
pub use std::ffi::c_void;

use frei0r_sys::*;
use std::ffi::CStr;

#[doc(hidden)]
pub unsafe extern "C" fn f0r_init() -> c_int {
    1
}

#[doc(hidden)]
pub unsafe extern "C" fn f0r_deinit() {}

#[doc(hidden)]
pub unsafe extern "C" fn f0r_get_plugin_info<P: Plugin>(info: *mut f0r_plugin_info_t) {
    let info = unsafe { &mut *info };
    let our_info = P::info();

    info.name = our_info.name.as_ptr();
    info.author = our_info.author.as_ptr();
    info.plugin_type = match our_info.plugin_type {
        PluginType::Filter => F0R_PLUGIN_TYPE_FILTER as i32,
        PluginType::Source => F0R_PLUGIN_TYPE_SOURCE as i32,
        PluginType::Mixer2 => F0R_PLUGIN_TYPE_MIXER2 as i32,
        PluginType::Mixer3 => F0R_PLUGIN_TYPE_MIXER3 as i32,
    };
    info.color_model = match our_info.color_model {
        ColorModel::BGRA8888 => F0R_COLOR_MODEL_BGRA8888 as i32,
        ColorModel::RGBA8888 => F0R_COLOR_MODEL_RGBA8888 as i32,
        ColorModel::PACKED32 => F0R_COLOR_MODEL_PACKED32 as i32,
    };
    info.frei0r_version = FREI0R_MAJOR_VERSION as i32;
    info.major_version = our_info.major_version;
    info.minor_version = our_info.minor_version;
    info.num_params = P::param_count().try_into().unwrap();
    info.explanation = our_info.explanation.as_ptr();
}

#[doc(hidden)]
pub unsafe fn f0r_get_param_info<P: Plugin>(info: *mut f0r_param_info_t, param_index: c_int) {
    let param_index = param_index.try_into().unwrap();

    let info = unsafe { &mut *info };
    let our_info = P::param_info(param_index);

    info.name = our_info.name.as_ptr();
    info.type_ = match our_info.kind {
        ParamKind::Bool => F0R_PARAM_BOOL as i32,
        ParamKind::Double => F0R_PARAM_DOUBLE as i32,
        ParamKind::Color => F0R_PARAM_COLOR as i32,
        ParamKind::Position => F0R_PARAM_POSITION as i32,
        ParamKind::String => F0R_PARAM_STRING as i32,
    };
    info.explanation = our_info.explanation.as_ptr();
}

pub struct Instance<P: Plugin> {
    frame_length: usize,
    plugin_type: PluginType,
    inner: P,
}

#[doc(hidden)]
pub unsafe extern "C" fn f0r_construct<P: Plugin>(width: c_uint, height: c_uint) -> f0r_instance_t {
    let width = width.try_into().unwrap();
    let height = height.try_into().unwrap();
    let instance = P::new(width, height);
    let instance = Instance {
        frame_length: width * height,
        plugin_type: P::info().plugin_type,
        inner: instance,
    };
    Box::into_raw(Box::new(instance)) as f0r_instance_t
}

#[doc(hidden)]
pub unsafe extern "C" fn f0r_destruct<P: Plugin>(instance: f0r_instance_t) {
    let instance = unsafe { Box::from_raw(instance as *mut Instance<P>) };
    drop(instance)
}

#[doc(hidden)]
pub unsafe extern "C" fn f0r_set_param_value<P: Plugin>(
    instance: f0r_instance_t,
    param: f0r_param_t,
    param_index: c_int,
) {
    let param_index = param_index.try_into().unwrap();

    let instance = unsafe { &mut *(instance as *mut Instance<P>) };
    match instance.inner.param_mut(param_index) {
        ParamMut::Bool(value) => {
            assert!(P::param_info(param_index).kind == ParamKind::Bool);

            let param = unsafe { *(param as *const f0r_param_bool) };
            *value = param >= 0.5;
        }
        ParamMut::Double(value) => {
            assert!(P::param_info(param_index).kind == ParamKind::Double);

            let param = unsafe { *(param as *const f0r_param_double) };
            *value = param;
        }
        ParamMut::Color(value) => {
            assert!(P::param_info(param_index).kind == ParamKind::Color);

            let param = unsafe { *(param as *const f0r_param_color) };
            value.r = param.r;
            value.g = param.g;
            value.b = param.b;
        }
        ParamMut::Position(value) => {
            assert!(P::param_info(param_index).kind == ParamKind::Position);

            let param = unsafe { *(param as *const f0r_param_position) };
            value.x = param.x;
            value.y = param.y;
        }
        ParamMut::String(value) => {
            assert!(P::param_info(param_index).kind == ParamKind::String);

            let param = unsafe { *(param as *const f0r_param_string) };
            *value = unsafe { CStr::from_ptr(param) }.to_owned();
        }
    };
}

#[doc(hidden)]
pub unsafe extern "C" fn f0r_get_param_value<P: Plugin>(
    instance: f0r_instance_t,
    param: f0r_param_t,
    param_index: c_int,
) {
    let param_index = param_index.try_into().unwrap();

    let instance = unsafe { &mut *(instance as *mut Instance<P>) };
    match instance.inner.param_ref(param_index) {
        ParamRef::Bool(value) => {
            assert!(P::param_info(param_index).kind == ParamKind::Bool);

            let param = unsafe { &mut *(param as *mut f0r_param_bool) };
            *param = if *value { 1.0 } else { 0.0 };
        }
        ParamRef::Double(value) => {
            assert!(P::param_info(param_index).kind == ParamKind::Double);

            let param = unsafe { &mut *(param as *mut f0r_param_double) };
            *param = *value;
        }
        ParamRef::Color(value) => {
            assert!(P::param_info(param_index).kind == ParamKind::Color);

            let param = unsafe { &mut *(param as *mut f0r_param_color) };
            param.r = value.r;
            param.g = value.g;
            param.b = value.b;
        }
        ParamRef::Position(value) => {
            assert!(P::param_info(param_index).kind == ParamKind::Position);

            let param = unsafe { &mut *(param as *mut f0r_param_position) };
            param.x = value.x;
            param.y = value.y;
        }
        ParamRef::String(value) => {
            assert!(P::param_info(param_index).kind == ParamKind::String);

            let param = unsafe { &mut *(param as *mut f0r_param_string) };
            // We are casting away constness here. This should be fine since quoting the
            // comment found in the original header, "If the caller needs to modify the
            // value, it should make a copy of it and modify before calling
            // f0r_set_param_value()."
            *param = value.as_ptr() as f0r_param_string;
        }
    };
}

fn frame_to_slice(frame: &*const u32, length: usize) -> &[u32] {
    if frame.is_null() {
        panic!("Unexpected null frame");
    } else {
        unsafe { std::slice::from_raw_parts(*frame, length) }
    }
}

#[doc(hidden)]
pub unsafe extern "C" fn f0r_update<P: Plugin>(
    instance: f0r_instance_t,
    time: f64,
    inframe: *const u32,
    outframe: *mut u32,
) {
    unsafe {
        f0r_update2::<P>(
            instance,
            time,
            inframe,
            std::ptr::null(),
            std::ptr::null(),
            outframe,
        )
    };
}

#[doc(hidden)]
pub unsafe extern "C" fn f0r_update2<P: Plugin>(
    instance: f0r_instance_t,
    time: f64,
    inframe1: *const u32,
    inframe2: *const u32,
    inframe3: *const u32,
    outframe: *mut u32,
) {
    let instance = unsafe { &mut *(instance as *mut Instance<P>) };
    if outframe.is_null() {
        panic!("unexpected null output frame");
    }
    let outframe = unsafe { std::slice::from_raw_parts_mut(outframe, instance.frame_length) };
    match instance.plugin_type {
        PluginType::Source => {
            instance.inner.source_update(time, outframe);
        }
        PluginType::Filter => {
            instance.inner.filter_update(
                time,
                frame_to_slice(&inframe1, instance.frame_length),
                outframe,
            );
        }
        PluginType::Mixer2 => {
            instance.inner.mixer2_update(
                time,
                frame_to_slice(&inframe1, instance.frame_length),
                frame_to_slice(&inframe2, instance.frame_length),
                outframe,
            );
        }
        PluginType::Mixer3 => {
            instance.inner.mixer3_update(
                time,
                frame_to_slice(&inframe1, instance.frame_length),
                frame_to_slice(&inframe2, instance.frame_length),
                frame_to_slice(&inframe3, instance.frame_length),
                outframe,
            );
        }
    }
}
