use crate::window::WindowHandle;
use crate::Rectangle;

#[derive(Copy, Clone, Debug)]
pub enum MouseButton {
    Left = 1,
    Middle = 2,
    Right = 3,
    ScrollUp = 4,
    ScrollDown = 5,
}

#[derive(Copy, Clone, Debug)]
pub enum Event<'a> {
    WindowCreated {
        window: WindowHandle<'a>,
    },
    WindowDestroyed {
        window: WindowHandle<'a>,
    },
    WindowConfigured {
        window: WindowHandle<'a>,
    },
    WindowMapped {
        window: WindowHandle<'a>,
    },
    WindowUnmapped {
        window: WindowHandle<'a>,
    },
    WindowConfigurationRequest {
        window: WindowHandle<'a>,
        rectangle: Rectangle,
    },
    WindowMappingRequest {
        window: WindowHandle<'a>,
    },
    WindowReparented {
        window: WindowHandle<'a>,
    },
    MotionNotify {
        window: WindowHandle<'a>,
        x: i16,
        y: i16,
    },
    ButtonPressed {
        root_window: WindowHandle<'a>,
        child_window: Option<WindowHandle<'a>>,
        button: MouseButton,
    },
    ButtonReleased {
        root_window: WindowHandle<'a>,
        child_window: Option<WindowHandle<'a>>,
        button: MouseButton,
    },
    Unknown,
}

bitflags! {
    pub struct EventMask : u32 {
        const NO_EVENT = 0;
        const KEY_PRESS = 1;
        const KEY_RELEASE = 2;
        const BUTTON_PRESS = 4;
        const BUTTON_RELEASE = 8;
        const ENTER_WINDOW = 16;
        const LEAVE_WINDOW = 32;
        const POINTER_MOTION = 64;
        const POINTER_MOTION_HINT = 128;
        const BUTTON_1_MOTION = 256;
        const BUTTON_2_MOTION = 512;
        const BUTTON_3_MOTION = 1024;
        const BUTTON_4_MOTION = 2048;
        const BUTTON_5_MOTION = 4096;
        const BUTTON_MOTION = 8192;
        const KEYMAP_STATE = 16_384;
        const EXPOSURE = 32_768;
        const VISIBILITY_CHANGE = 65_536;
        const STRUCTURE_NOTIFY = 131_072;
        const RESIZE_REDIRECT = 262_144;
        const SUBSTRUCTURE_NOTIFY = 524_288;
        const SUBSTRUCTURE_REDIRECT = 1_048_576;
        const FOCUS_CHANGE = 2_097_152;
        const PROPERTY_CHANGE = 4_194_304;
        const COLOR_MAP_CHANGE = 8_388_608;
        const OWNER_GRAB_BUTTON = 16_777_216;
    }
}
