use std::os::raw::c_void;
use std::ptr;
use xcb_system::{
    xcb_connect, xcb_connection_has_error, xcb_connection_t, xcb_disconnect, xcb_get_setup,
    xcb_setup_t, xcb_window_t,
};

#[derive(Debug)]
pub struct XcbConnection {
    connection: *mut xcb_connection_t,
    setup: *const xcb_setup_t,
    default_screen: u32,
}

#[derive(Debug)]
pub struct XcbWindow<'a> {
    connection: &'a XcbConnection,
    window: xcb_window_t,
}

#[derive(Debug)]
pub enum XcbError {
    ConnectionFailed,
    UnsupportedExtension,
    InsufficientMemory,
    RequestLengthExceeded,
    DisplayParseError,
    InvalidScreen,
    UnknownError(u32),
    ScreenNotFound(u32),
}

impl XcbConnection {
    pub fn new() -> Result<Self, XcbError> {
        unsafe {
            let mut default_screen: i32 = 0;
            let connection = xcb_connect(ptr::null(), &mut default_screen);

            match xcb_connection_has_error(connection) as u32 {
                0 => {}
                xcb_system::XCB_CONN_ERROR => return Err(XcbError::ConnectionFailed),
                xcb_system::XCB_CONN_CLOSED_EXT_NOTSUPPORTED => {
                    return Err(XcbError::UnsupportedExtension)
                }
                xcb_system::XCB_CONN_CLOSED_MEM_INSUFFICIENT => {
                    return Err(XcbError::InsufficientMemory)
                }
                xcb_system::XCB_CONN_CLOSED_REQ_LEN_EXCEED => {
                    return Err(XcbError::RequestLengthExceeded)
                }
                xcb_system::XCB_CONN_CLOSED_PARSE_ERR => return Err(XcbError::DisplayParseError),
                xcb_system::XCB_CONN_CLOSED_INVALID_SCREEN => return Err(XcbError::InvalidScreen),
                error => return Err(XcbError::UnknownError(error)),
            }

            let setup = xcb_get_setup(connection);

            Ok(XcbConnection {
                connection,
                setup,
                default_screen: default_screen as u32,
            })
        }
    }

    pub fn get_vendor(&self) -> String {
        let length = unsafe { xcb_system::xcb_setup_vendor_length(self.setup) } as usize;
        let vendor = unsafe { xcb_system::xcb_setup_vendor(self.setup) };

        let mut buf: Vec<u8> = vec![0; length + 1];

        unsafe {
            std::ptr::copy_nonoverlapping(vendor, buf.as_mut_ptr() as *mut i8, length);
        };

        String::from_utf8(buf).unwrap()
    }

    pub fn get_root_window(&self) -> XcbWindow {
        let screen = self.get_screen(self.default_screen).unwrap();

        XcbWindow {
            window: (screen).root,
            connection: self,
        }
    }

    pub(crate) fn get_connection(&self) -> *mut xcb_connection_t {
        self.connection
    }

    fn get_screen(&self, screen_number: u32) -> Result<xcb_system::xcb_screen_t, XcbError> {
        let mut iterator = unsafe { xcb_system::xcb_setup_roots_iterator(self.setup) };

        for _ in 0..screen_number {
            unsafe { xcb_system::xcb_screen_next(&mut iterator) };

            if iterator.rem == 0 {
                return Err(XcbError::ScreenNotFound(screen_number));
            }
        }

        if iterator.data.is_null() {
            return Err(XcbError::ScreenNotFound(screen_number));
        }

        Ok(unsafe { *iterator.data })
    }
}

impl Drop for XcbConnection {
    fn drop(&mut self) {
        unsafe {
            xcb_disconnect(self.connection);
        }
    }
}

#[derive(Copy, Clone)]
pub enum Event {
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
    EXPOSURE = 32_768,
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

impl XcbWindow<'_> {
    pub fn set_event_mask(&self, events: Vec<Event>) {
        let converted_events: Vec<u32> = events.iter().map(|event| *event as u32).collect();
        unsafe {
            xcb_system::xcb_change_window_attributes(
                self.connection.get_connection(),
                self.window,
                xcb_system::xcb_cw_t_XCB_CW_EVENT_MASK,
                converted_events.as_ptr() as *const c_void,
            );
        }
    }
}
