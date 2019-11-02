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

#[derive(Copy, Clone, Debug)]
pub struct WindowHandle(u32);

#[derive(Copy, Clone, Debug)]
pub enum XcbEvent {
    WindowCreated { window: WindowHandle },
    WindowDestroyed { window: WindowHandle },
    WindowConfigured { window: WindowHandle },
    WindowMapped { window: WindowHandle },
    WindowUnmapped { window: WindowHandle },
    WindowConfigurationRequest { window: WindowHandle, x: i16, y: i16, width: u16, height: u16 },
    WindowMappingRequest { window: WindowHandle },
    Unknown
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

    pub fn wait_for_event(&self) -> XcbEvent {
        let event_ptr = unsafe { xcb_system::xcb_wait_for_event(self.connection) };
        // todo handle null result?
        let event = unsafe { *event_ptr };

        match u32::from(event.response_type & !0x80) {
            xcb_system::XCB_CREATE_NOTIFY => {
                let create_notify_event = unsafe { *(event_ptr as *const xcb_system::xcb_create_notify_event_t) };

                XcbEvent::WindowCreated { window: WindowHandle(create_notify_event.window) }
            }
            xcb_system::XCB_DESTROY_NOTIFY => {
                let destroy_notify_event = unsafe { *(event_ptr as *const xcb_system::xcb_destroy_notify_event_t) };

                XcbEvent::WindowDestroyed { window: WindowHandle(destroy_notify_event.window) }
            }
            xcb_system::XCB_CONFIGURE_NOTIFY => {
                let configure_notify_event = unsafe { *(event_ptr as *const xcb_system::xcb_configure_notify_event_t) };

                XcbEvent::WindowConfigured { window: WindowHandle(configure_notify_event.window) }
            }
            xcb_system::XCB_UNMAP_NOTIFY => {
                let unmap_notify_event = unsafe { *(event_ptr as *const xcb_system::xcb_unmap_notify_event_t) };

                XcbEvent::WindowUnmapped { window: WindowHandle(unmap_notify_event.window) }
            }
            xcb_system::XCB_MAP_NOTIFY => {
                let map_notify_event = unsafe { *(event_ptr as *const xcb_system::xcb_map_notify_event_t) };

                XcbEvent::WindowMapped { window: WindowHandle(map_notify_event.window) }
            }
            xcb_system::XCB_CONFIGURE_REQUEST => {
                let configure_request = unsafe { *(event_ptr as *const xcb_system::xcb_configure_request_event_t) };

                XcbEvent::WindowConfigurationRequest {
                    window: WindowHandle(configure_request.window),
                    x: (configure_request.x),
                    y: (configure_request.y),
                    width: (configure_request.width),
                    height: (configure_request.height)
                }
            }
            xcb_system::XCB_MAP_REQUEST => {
                let map_request = unsafe { *(event_ptr as *const xcb_system::xcb_map_request_event_t) };

                XcbEvent::WindowMappingRequest { window: WindowHandle(map_request.window) }
            }
            _ => {
                println!("Unknown event: {:?}", event);

                XcbEvent::Unknown
            }
        }
    }

    pub(crate) fn get_connection(&self) -> *mut xcb_connection_t {
        self.connection
    }

    pub fn configure_window(&self, window:WindowHandle, x:i16, y:i16, width:u16, height:u16) -> XcbResult<()> {
        let values = vec![x as u32, y as u32, u32::from(width), u32::from(height)];
        let result = unsafe { xcb_system::xcb_configure_window(
            self.connection,
            window.0,
            (xcb_system::xcb_config_window_t_XCB_CONFIG_WINDOW_X
                | xcb_system::xcb_config_window_t_XCB_CONFIG_WINDOW_Y
                | xcb_system::xcb_config_window_t_XCB_CONFIG_WINDOW_WIDTH
                | xcb_system::xcb_config_window_t_XCB_CONFIG_WINDOW_HEIGHT) as u16,
            values.as_ptr() as *const c_void
        ) };

        XcbResult::new(result, &self)
    }

    pub fn map_window(&self, window:WindowHandle) -> XcbResult<()> {
        let result = unsafe {
            xcb_system::xcb_map_window(self.connection, window.0)
        };

        XcbResult::new(result, &self)
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
    pub fn set_event_mask(&self, events: Vec<EventMask>) -> XcbResult<()> {
        let mut mask = 0;
        for e in events {
            mask |= e as u32;
        }

        unsafe {
            let cookie = xcb_system::xcb_change_window_attributes(
                self.connection.get_connection(),
                self.window,
                xcb_system::xcb_cw_t_XCB_CW_EVENT_MASK,
                &mask as *const u32 as *const c_void,
            );

            XcbResult::new(cookie, self.connection)
        }
    }
}

pub struct XcbResult<'a, T> {
    cookie: xcb_system::xcb_void_cookie_t,
    connection: &'a XcbConnection,
    _marker: std::marker::PhantomData<T>
}

impl<'a> XcbResult<'a, ()> {
    pub fn new(cookie:xcb_system::xcb_void_cookie_t, connection:&'a XcbConnection) -> Self {
        Self {
            cookie,
            connection,
            _marker: std::marker::PhantomData {}
        }
    }

    pub fn get_result(&mut self) -> Result<(), XcbError> {
        let result = unsafe {
            xcb_system::xcb_request_check(
                self.connection.get_connection(),
                self.cookie
            )
        };

        if result.is_null() {
            Ok(())
        } else {
            println!("{:?}", unsafe { *result });
            Err(XcbError::UnknownError(1234)) // fixme get the actual error here
        }
    }
}