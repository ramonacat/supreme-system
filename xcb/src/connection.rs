use crate::event::{Event, MouseButton};
use crate::result::{Error, XcbResult};
use crate::window::{Window, WindowHandle};
use crate::Rectangle;
use xcb_system::{
    xcb_connect, xcb_connection_has_error, xcb_connection_t, xcb_disconnect, xcb_get_setup,
    xcb_setup_t,
};

#[derive(Debug)]
pub struct Connection {
    connection: *mut xcb_connection_t,
    setup: *const xcb_setup_t,
    default_screen: u32,
}

impl Connection {
    pub fn new() -> Result<Self, Error> {
        unsafe {
            let mut default_screen: i32 = 0;
            let connection = xcb_connect(std::ptr::null(), &mut default_screen);

            match xcb_connection_has_error(connection) as u32 {
                0 => {}
                xcb_system::XCB_CONN_ERROR => return Err(Error::ConnectionFailed),
                xcb_system::XCB_CONN_CLOSED_EXT_NOTSUPPORTED => {
                    return Err(Error::UnsupportedExtension)
                }
                xcb_system::XCB_CONN_CLOSED_MEM_INSUFFICIENT => {
                    return Err(Error::InsufficientMemory)
                }
                xcb_system::XCB_CONN_CLOSED_REQ_LEN_EXCEED => {
                    return Err(Error::RequestLengthExceeded)
                }
                xcb_system::XCB_CONN_CLOSED_PARSE_ERR => return Err(Error::DisplayParseError),
                xcb_system::XCB_CONN_CLOSED_INVALID_SCREEN => return Err(Error::InvalidScreen),
                error => return Err(Error::UnknownError(error)),
            }

            let setup = xcb_get_setup(connection);

            Ok(Connection {
                connection,
                setup,
                default_screen: default_screen as u32,
            })
        }
    }

    pub fn get_vendor(&self) -> Result<String, Error> {
        let length = unsafe { xcb_system::xcb_setup_vendor_length(self.setup) } as usize;
        let vendor = unsafe { xcb_system::xcb_setup_vendor(self.setup) };

        let mut buf: Vec<u8> = vec![0; length + 1];

        unsafe {
            std::ptr::copy_nonoverlapping(vendor, buf.as_mut_ptr() as *mut i8, length);
        };

        Ok(String::from_utf8(buf)?)
    }

    pub fn get_root_window(&self) -> Result<WindowHandle, Error> {
        let screen = self.get_screen(self.default_screen)?;

        Ok(WindowHandle::new(screen.root, &self))
    }

    pub(crate) fn get_root_visual(&self) -> Result<xcb_system::xcb_visualid_t, Error> {
        Ok(self.get_screen(self.default_screen)?.root_visual)
    }

    pub fn wait_for_event(&self) -> Event {
        let event_ptr = unsafe { xcb_system::xcb_wait_for_event(self.connection) };

        if event_ptr.is_null() {
            panic!("failed to wait for event");
        }

        let event = unsafe { *event_ptr };

        match u32::from(event.response_type & !0x80) {
            xcb_system::XCB_CREATE_NOTIFY => {
                let create_notify_event =
                    unsafe { *(event_ptr as *const xcb_system::xcb_create_notify_event_t) };

                Event::WindowCreated {
                    window: WindowHandle::new(create_notify_event.window, &self),
                }
            }
            xcb_system::XCB_DESTROY_NOTIFY => {
                let destroy_notify_event =
                    unsafe { *(event_ptr as *const xcb_system::xcb_destroy_notify_event_t) };

                Event::WindowDestroyed {
                    window: WindowHandle::new(destroy_notify_event.window, &self),
                }
            }
            xcb_system::XCB_CONFIGURE_NOTIFY => {
                let configure_notify_event =
                    unsafe { *(event_ptr as *const xcb_system::xcb_configure_notify_event_t) };

                Event::WindowConfigured {
                    window: WindowHandle::new(configure_notify_event.window, &self),
                }
            }
            xcb_system::XCB_UNMAP_NOTIFY => {
                let unmap_notify_event =
                    unsafe { *(event_ptr as *const xcb_system::xcb_unmap_notify_event_t) };

                Event::WindowUnmapped {
                    window: WindowHandle::new(unmap_notify_event.window, &self),
                }
            }
            xcb_system::XCB_MAP_NOTIFY => {
                let map_notify_event =
                    unsafe { *(event_ptr as *const xcb_system::xcb_map_notify_event_t) };

                Event::WindowMapped {
                    window: WindowHandle::new(map_notify_event.window, &self),
                }
            }
            xcb_system::XCB_CONFIGURE_REQUEST => {
                let configure_request =
                    unsafe { *(event_ptr as *const xcb_system::xcb_configure_request_event_t) };

                Event::WindowConfigurationRequest {
                    window: WindowHandle::new(configure_request.window, &self),
                    rectangle: Rectangle {
                        x: (configure_request.x),
                        y: (configure_request.y),
                        width: (configure_request.width),
                        height: (configure_request.height),
                    },
                }
            }
            xcb_system::XCB_MAP_REQUEST => {
                let map_request =
                    unsafe { *(event_ptr as *const xcb_system::xcb_map_request_event_t) };

                Event::WindowMappingRequest {
                    window: WindowHandle::new(map_request.window, &self),
                }
            }
            xcb_system::XCB_REPARENT_NOTIFY => {
                let reparent =
                    unsafe { *(event_ptr as *const xcb_system::xcb_reparent_notify_event_t) };

                Event::WindowReparented {
                    window: WindowHandle::new(reparent.window, &self),
                }
            }
            xcb_system::XCB_MOTION_NOTIFY => {
                let motion_notify =
                    unsafe { *(event_ptr as *const xcb_system::xcb_motion_notify_event_t) };

                println!("Motion notify: {:?}", motion_notify);

                Event::MotionNotify {
                    window: WindowHandle::new(motion_notify.event, &self),
                    x: motion_notify.root_x,
                    y: motion_notify.root_y,
                }
            }
            xcb_system::XCB_BUTTON_PRESS => {
                let button_press =
                    unsafe { *(event_ptr as *const xcb_system::xcb_button_press_event_t) };

                Event::ButtonPressed {
                    root_window: WindowHandle::new(button_press.root, &self),
                    child_window: if button_press.child == 0 {
                        None
                    } else {
                        Some(WindowHandle::new(button_press.child, &self))
                    },
                    button: match button_press.detail {
                        1 => MouseButton::Left,
                        2 => MouseButton::Middle,
                        3 => MouseButton::Right,
                        4 => MouseButton::ScrollUp,
                        5 => MouseButton::ScrollDown,
                        _ => panic!("Unknown mouse button {}", button_press.detail),
                    },
                }
            }
            xcb_system::XCB_BUTTON_RELEASE => {
                let button_release =
                    unsafe { *(event_ptr as *const xcb_system::xcb_button_release_event_t) };

                Event::ButtonReleased {
                    root_window: WindowHandle::new(button_release.root, &self),
                    child_window: if button_release.child == 0 {
                        None
                    } else {
                        Some(WindowHandle::new(button_release.child, &self))
                    },
                    button: match button_release.detail {
                        1 => MouseButton::Left,
                        2 => MouseButton::Middle,
                        3 => MouseButton::Right,
                        4 => MouseButton::ScrollUp,
                        5 => MouseButton::ScrollDown,
                        _ => panic!("Unknown mouse button {}", button_release.detail),
                    },
                }
            }
            _ => {
                println!("Unknown event: {:?}", event);

                Event::Unknown
            }
        }
    }

    pub fn grab_pointer(&self) -> XcbResult<xcb_system::xcb_grab_pointer_reply_t, bool> {
        let cookie = unsafe {
            xcb_system::xcb_grab_pointer(
                self.connection,
                0,
                self.get_root_window().unwrap().id(),
                64 | 8, // todo this is pointer move | button release, use the EventMask enum instead
                xcb_system::xcb_grab_mode_t_XCB_GRAB_MODE_ASYNC as u8,
                xcb_system::xcb_grab_mode_t_XCB_GRAB_MODE_ASYNC as u8,
                xcb_system::XCB_NONE,
                xcb_system::XCB_NONE,
                xcb_system::XCB_CURRENT_TIME,
            )
        };

        XcbResult::new(
            Box::new(move |connection: &Connection| {
                let mut error: *mut xcb_system::xcb_generic_error_t = std::ptr::null_mut();

                let reply = unsafe {
                    *xcb_system::xcb_grab_pointer_reply(
                        connection.get_connection(),
                        cookie,
                        &mut error,
                    )
                };

                (reply, error)
            }),
            Box::new(|reply| {
                reply.status == xcb_system::xcb_grab_status_t_XCB_GRAB_STATUS_SUCCESS as u8
            }),
            &self,
        )
    }

    pub fn ungrab_pointer(&self) -> XcbResult<(), ()> {
        let cookie = unsafe {
            xcb_system::xcb_ungrab_pointer_checked(self.connection, xcb_system::XCB_CURRENT_TIME)
        };

        XcbResult::new_void(cookie, &self)
    }

    pub(crate) fn get_connection(&self) -> *mut xcb_connection_t {
        self.connection
    }

    fn get_screen(&self, screen_number: u32) -> Result<xcb_system::xcb_screen_t, Error> {
        let mut iterator = unsafe { xcb_system::xcb_setup_roots_iterator(self.setup) };

        for _ in 0..screen_number {
            unsafe { xcb_system::xcb_screen_next(&mut iterator) };

            if iterator.rem == 0 {
                return Err(Error::ScreenNotFound(screen_number));
            }
        }

        if iterator.data.is_null() {
            return Err(Error::ScreenNotFound(screen_number));
        }

        Ok(unsafe { *iterator.data })
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            xcb_disconnect(self.connection);
        }
    }
}
