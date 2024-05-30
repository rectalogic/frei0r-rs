pub mod ffi;

use std::ffi::CStr;
use std::ffi::CString;

/// Type of the plugin
///
/// These defines determine whether the plugin is a source, a filter or one of the two mixer types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginType {
    /// One input and one output
    Filter,
    /// Just one output
    Source,
    /// Two inputs and one output
    Mixer2,
    /// Three inputs and one output
    Mixer3,
}

/// List of supported color models.
///
/// Note: the color models are endian independent, because the color components are defined by
/// their positon in memory, not by their significance in an uint32_t value.
///
/// For effects that work on the color components, RGBA8888 is the recommended color model for
/// frei0r-1.2 effects. For effects that only work on pixels, PACKED32 is the recommended color
/// model since it helps the application to avoid unnecessary color conversions.
///
/// Effects can choose an appropriate color model, applications must support all color models and
/// do conversions if necessary. Source effects must not use the PACKED32 color model because the
/// application must know in which color model the created framebuffers are represented.
///
/// For each color model, a frame consists of width*height pixels which are stored row-wise and
/// consecutively in memory. The size of a pixel is
/// 4 bytes. There is no extra pitch parameter (i.e. the pitch is simply width*4).
///
/// The following additional constraints must be honored: - The top-most line of a frame is stored
/// first in memory. - A frame must be aligned to a 16 byte border in memory. - The width and
/// height of a frame must be positive - The width and height of a frame must be integer multiples
/// of 8
///
/// These constraints make sure that each line is stored at an address aligned to 16 byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorModel {
    /// In BGRA8888, each pixel is represented by 4 consecutive unsigned bytes, where the first
    /// byte value represents the blue, the second the green, and the third the red color component
    /// of the pixel. The last value represents the alpha value.
    BGRA8888,

    /// In RGBA8888, each pixel is represented by 4 consecutive unsigned bytes, where the first
    /// byte value represents the red, the second the green, and the third the blue color component
    /// of the pixel. The last value represents the alpha value.
    RGBA8888,

    /// In PACKED32, each pixel is represented by 4 consecutive bytes, but it is not defined how
    /// the color componets are stored. The true color format could be RGBA8888, BGRA8888, a packed
    /// 32 bit YUV format, or any other color format that stores pixels in 32 bit.
    ///
    /// This is useful for effects that don't work on color but only on pixels (for example a
    /// mirror effect).
    ///
    /// Note that source effects must not use this color model.
    PACKED32,
}

/// PluginInfo is returned by the plugin to tell the application about its name, type, number of
/// parameters, and version.
///
/// An application should ignore (i.e. not use) frei0r effects that have unknown values in the
/// plugin_type or color_model field. It should also ignore effects with a too high frei0r_version.
///
/// This is necessary to be able to extend the frei0r spec (e.g. by adding new color models or
/// plugin types) in a way that does not result in crashes when loading effects that make use of
/// these extensions into an older application.
#[derive(Debug, Clone, Copy)]
pub struct PluginInfo {
    /// The (short) name of the plugin
    pub name : &'static CStr,
    /// The plugin author
    pub author : &'static CStr,
    /// The plugin type
    pub plugin_type : PluginType,
    /// The color model used
    pub color_model : ColorModel,
    /// The major version of the plugin
    pub major_version : i32,
    /// The minor version of the plugin
    pub minor_version : i32,
    /// The number of parameters of the plugin
    pub num_params : usize,
    /// An optional explanation string
    pub explanation : &'static CStr,
}

/// Parameter types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    /// Booleans
    Bool,
    /// Doubles
    Double,
    /// Color
    Color,
    /// Position
    Position,
    /// String
    String,
}

/// Type returned by plugin for every parameter.
#[derive(Debug, Clone, Copy)]
pub struct ParamInfo {
    pub name : &'static CStr,
    pub param_type : ParamType,
    pub explanation : &'static CStr,
}

/// Parameters
#[derive(Debug, Clone)]
pub enum Param {
    /// Booleans
    Bool(bool),
    /// Doubles
    Double(f64),
    /// Color
    Color { r : f32, g : f32, b : f32 },
    /// Position
    Position { x : f64, y : f64, },
    /// String
    String(CString),
}

pub trait Plugin {
    /// Called by the application to query plugin information.
    fn info() -> PluginInfo;

    /// Called by the application to query the type of each parameter.
    fn param_info(index : usize) -> ParamInfo;

    /// Constructor for effect instances.
    ///
    /// The resolution must be an integer multiple of 8, must be greater than 0 and be at most 2048
    /// in both dimensions.
    ///
    /// The plugin must set default values for all parameters in this function.
    fn new(width : usize, height : usize) -> Self;

    /// Return a reference to parameter.
    fn param(&self, param_index : usize) -> &Param;

    /// Return a mutable reference to parameter.
    fn param_mut(&mut self, param_index : usize) -> &mut Param;

    ///// This function allows the application to set the parameter values of an effect instance.
    //fn param(&self, param_index : usize) -> Param;

    ///// This function allows the application to query the parameter values of an effect instance.
    //fn set_param<'a>(&mut self, param_index : usize, param : Param<'a>);

    /// This is where the core effect processing happens. The application calls it after it has
    /// set the necessary parameter values.
    ///
    /// This function should not alter the parameters of the effect in any way ([Instance::param]
    /// should return the same values after a call to [Instance::update] as before the call).
    ///
    /// The function is responsible to restore the fpu state (e.g. rounding mode) and mmx state if
    /// applicable before it returns to the caller.
    ///
    /// This is never called for effect of type [PluginType::Mixer2] and [PluginType::Mixer3].
    fn update(&self, time : f64, width : usize, height : usize, inframe : &[u32], outframe : &mut [u32]);

    /// For effect of type [PluginType::Mixer2] and [PluginType::Mixer3].
    fn update2(&self, time : f64, width : usize, height : usize, inframe1 : &[u32], inframe2 : &[u32], inframe3 : &[u32], outframe : &mut [u32]);
}

#[macro_export]
macro_rules! plugin {
    ($type:ty) => {
        use frei0r_rs::ffi;

        #[no_mangle] pub unsafe extern "C" fn f0r_init() -> ffi::c_int { ffi::f0r_init() }
        #[no_mangle] pub unsafe extern "C" fn f0r_deinit() { ffi::f0r_deinit()  }
        #[no_mangle] pub unsafe extern "C" fn f0r_get_plugin_info(info: *mut ffi::f0r_plugin_info_t) { ffi::f0r_get_plugin_info::<$type>(info)  }
        #[no_mangle] pub unsafe extern "C" fn f0r_get_param_info(info: *mut ffi::f0r_param_info_t, param_index: ffi::c_int) { ffi::f0r_get_param_info::<$type>(info, param_index) }
        #[no_mangle] pub unsafe extern "C" fn f0r_construct(width : ffi::c_uint, height: ffi::c_uint) -> ffi::f0r_instance_t { ffi::f0r_construct::<$type>(width, height) }
        #[no_mangle] pub unsafe extern "C" fn f0r_destruct(instance : ffi::f0r_instance_t) { ffi::f0r_destruct::<$type>(instance) }
        #[no_mangle] pub unsafe extern "C" fn f0r_set_param_value(instance: ffi::f0r_instance_t, param: ffi::f0r_param_t, param_index: ffi::c_int) { ffi::f0r_set_param_value::<$type>(instance, param, param_index)  }
        #[no_mangle] pub unsafe extern "C" fn f0r_get_param_value(instance: ffi::f0r_instance_t, param: ffi::f0r_param_t, param_index: ffi::c_int) { ffi::f0r_get_param_value::<$type>(instance, param, param_index)  }
        #[no_mangle] pub unsafe extern "C" fn f0r_update(instance: ffi::f0r_instance_t, time: f64, inframe: *const u32, outframe: *mut u32) { ffi::f0r_update::<$type>(instance, time, inframe, outframe) }
        #[no_mangle] pub unsafe extern "C" fn f0r_update2(instance: ffi::f0r_instance_t, time: f64, inframe1: *const u32, inframe2: *const u32, inframe3: *const u32, outframe: *mut u32) { ffi::f0r_update2::<$type>(instance, time, inframe1, inframe2, inframe3, outframe) }
    }
}
