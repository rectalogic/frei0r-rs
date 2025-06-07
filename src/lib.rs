//! Rust binding for the implementation of fri0r plugin API <https://frei0r.dyne.org/>.
//!
//! See example for API usage.

#[doc(hidden)]
pub mod ffi;
mod param;
pub use ffi::{KindFilter, KindMixer2, KindMixer3, KindSource, PluginKind};
pub use param::{Color, ParamInfo, ParamKind, Position};
use std::ffi::CStr;

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

/// The type returned by the plugin to tell the application about its name, type, number of
/// parameters, and version.
#[derive(Debug, Clone, Copy)]
pub struct PluginInfo {
    /// The (short) name of the plugin
    pub name: &'static CStr,
    /// The plugin author
    pub author: &'static CStr,
    /// The color model used
    pub color_model: ColorModel,
    /// The major version of the plugin
    pub major_version: i32,
    /// The minor version of the plugin
    pub minor_version: i32,
    /// An optional explanation string
    pub explanation: Option<&'static CStr>,
}

/// The plugin base trait. Plugins must also implement one of the
/// [SourcePlugin], [FilterPlugin], [Mixer2Plugin] or [Mixer3Plugin] traits
/// corresponding to the [PluginKind] associated type.
///
/// The update functions are where the core effect processing happens. The application calls it after it has
/// set the necessary parameter values.
///
/// The function is responsible to restore the fpu state (e.g. rounding mode) and mmx state if
/// applicable before it returns to the caller.
pub trait Plugin: 'static + Sized {
    type Kind: PluginKind;

    /// The list of plugin parameters
    const PARAMS: &'static [ParamInfo<Self>];

    /// Called by the application to query plugin information.
    fn info() -> PluginInfo;

    /// Constructor for effect instances.
    ///
    /// The resolution must be an integer multiple of 8, must be greater than 0 and be at most 2048
    /// in both dimensions.
    ///
    /// The plugin must set default values for all parameters in this function.
    fn new(width: usize, height: usize) -> Self;
}

/// A source plugin, must be implemented if Plugin::Kind = KindSource
pub trait SourcePlugin: Plugin<Kind = KindSource> {
    fn update_source(&mut self, time: f64, outframe: &mut [u32]);
}

/// A filter plugin, must be implemented if Plugin::Kind = KindFilter
pub trait FilterPlugin: Plugin<Kind = KindFilter> {
    fn update_filter(&mut self, time: f64, inframe: &[u32], outframe: &mut [u32]);
}

/// A mixer2 plugin, must be implemented if Plugin::Kind = KindMixer2
pub trait Mixer2Plugin: Plugin<Kind = KindMixer2> {
    fn update_mixer2(
        &mut self,
        time: f64,
        inframe1: &[u32],
        inframe2: &[u32],
        outframe: &mut [u32],
    );
}

/// A mixer3 plugin, must be implemented if Plugin::Kind = KindMixer3
pub trait Mixer3Plugin: Plugin<Kind = KindMixer3> {
    fn update_mixer3(
        &mut self,
        time: f64,
        inframe1: &[u32],
        inframe2: &[u32],
        inframe3: &[u32],
        outframe: &mut [u32],
    );
}

/// Export necessary C bindings for frei0r plugin.
#[macro_export]
macro_rules! plugin {
    ($type:ty) => {
        use frei0r_rs2::ffi;

        #[unsafe(no_mangle)]
        pub extern "C" fn f0r_init() -> ffi::c_int {
            ffi::f0r_init()
        }
        #[unsafe(no_mangle)]
        pub extern "C" fn f0r_deinit() {
            ffi::f0r_deinit()
        }
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn f0r_get_plugin_info(info: *mut ffi::f0r_plugin_info_t) {
            unsafe { ffi::f0r_get_plugin_info::<$type>(info) }
        }
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn f0r_get_param_info(
            info: *mut ffi::f0r_param_info_t,
            param_index: ffi::c_int,
        ) {
            unsafe { ffi::f0r_get_param_info::<$type>(info, param_index) }
        }
        #[unsafe(no_mangle)]
        pub extern "C" fn f0r_construct(
            width: ffi::c_uint,
            height: ffi::c_uint,
        ) -> ffi::f0r_instance_t {
            ffi::f0r_construct::<$type>(width, height)
        }
        #[unsafe(no_mangle)]
        pub extern "C" fn f0r_destruct(instance: ffi::f0r_instance_t) {
            ffi::f0r_destruct::<$type>(instance)
        }
        #[unsafe(no_mangle)]
        pub extern "C" fn f0r_set_param_value(
            instance: ffi::f0r_instance_t,
            param: ffi::f0r_param_t,
            param_index: ffi::c_int,
        ) {
            ffi::f0r_set_param_value::<$type>(instance, param, param_index)
        }
        #[unsafe(no_mangle)]
        pub extern "C" fn f0r_get_param_value(
            instance: ffi::f0r_instance_t,
            param: ffi::f0r_param_t,
            param_index: ffi::c_int,
        ) {
            ffi::f0r_get_param_value::<$type>(instance, param, param_index)
        }
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn f0r_update(
            instance: ffi::f0r_instance_t,
            time: f64,
            inframe: *const u32,
            outframe: *mut u32,
        ) {
            unsafe {
                ffi::f0r_update2::<$type>(
                    instance,
                    time,
                    inframe,
                    std::ptr::null(),
                    std::ptr::null(),
                    outframe,
                )
            }
        }
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn f0r_update2(
            instance: ffi::f0r_instance_t,
            time: f64,
            inframe1: *const u32,
            inframe2: *const u32,
            inframe3: *const u32,
            outframe: *mut u32,
        ) {
            unsafe {
                ffi::f0r_update2::<$type>(instance, time, inframe1, inframe2, inframe3, outframe)
            }
        }
    };
}
