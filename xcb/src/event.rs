use crate::window::WindowHandle;

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
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    },
    WindowMappingRequest {
        window: WindowHandle<'a>,
    },
    Unknown,
}

#[derive(Copy, Clone)]
pub enum EventMask {
    NoEvent = 0,
    KeyPress = 1,
    KeyRelease = 2,
    ButtonPress = 4,
    ButtonRelease = 8,
    EnterWindow = 16,
    LeaveWindow = 32,
    PointerMotion = 64,
    PointerMotionHint = 128,
    Button1Motion = 256,
    Button2Motion = 512,
    Button3Motion = 1024,
    Button4Motion = 2048,
    Button5Motion = 4096,
    ButtonMotion = 8192,
    KeymapState = 16_384,
    Exposure = 32_768,
    VisibilityChange = 65_536,
    StructureNotify = 131_072,
    ResizeRedirect = 262_144,
    SubstructureNotify = 524_288,
    SubstructureRedirect = 1_048_576,
    FocusChange = 2_097_152,
    PropertyChange = 4_194_304,
    ColorMapChange = 8_388_608,
    OwnerGrabButton = 16_777_216,
}
