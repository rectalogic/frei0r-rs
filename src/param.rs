use std::ffi::CStr;

/// Color parameter.
///
/// All components are in the range [0, 1].
#[derive(Debug, Clone, Copy)]
pub struct Color {
    /// Red component.
    pub r: f32,
    /// Green component.
    pub g: f32,
    /// Blue component.
    pub b: f32,
}

/// Position parameter.
///
/// All coordinates are in the range [0, 1].
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Type of a parameter.
#[derive(Debug)]
pub enum ParamKind<T> {
    Bool {
        get: fn(&T) -> bool,
        set: fn(&mut T, bool),
    },
    Double {
        get: fn(&T) -> f64,
        set: fn(&mut T, f64),
    },
    Color {
        get: fn(&T) -> &Color,
        set: fn(&mut T, &Color),
    },
    Position {
        get: fn(&T) -> &Position,
        set: fn(&mut T, &Position),
    },
    String {
        get: fn(&T) -> &CStr,
        set: fn(&mut T, &CStr),
    },
}

macro_rules! param_info_new {
    ($type:ident, $rust_get_type:ty, $rust_set_type:ty) => {
        paste::paste! {
            #[doc = "Create a " $type " parameter"]
            pub const fn [<new_ $type>](
                name: &'static CStr,
                explanation: &'static CStr,
                get: fn(&T) -> $rust_get_type,
                set: fn(&mut T, $rust_set_type),
            ) -> Self {
                ParamInfo {
                    name,
                    explanation,
                    kind: ParamKind::[<$type:camel>] { get, set },
                }
            }
        }
    };
}

/// Information about a parameter.
#[derive(Debug)]
pub struct ParamInfo<T> {
    name: &'static CStr,
    explanation: &'static CStr,
    kind: ParamKind<T>,
}

impl<T> ParamInfo<T> {
    param_info_new!(bool, bool, bool);
    param_info_new!(double, f64, f64);
    param_info_new!(color, &Color, &Color);
    param_info_new!(position, &Position, &Position);
    param_info_new!(string, &CStr, &CStr);

    pub(crate) fn name(&self) -> &'static CStr {
        self.name
    }

    pub(crate) fn explanation(&self) -> &'static CStr {
        self.explanation
    }

    pub(crate) fn kind(&self) -> &ParamKind<T> {
        &self.kind
    }
}
