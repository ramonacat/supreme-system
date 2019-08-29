use std::ptr;
use xcb_system::{xcb_connect, xcb_connection_t, xcb_disconnect, xcb_get_setup, xcb_setup_t};

pub struct XcbConnection {
    xcb_connection: *mut xcb_connection_t,
}

impl XcbConnection {
    pub fn new() -> Self {
        unsafe {
            let connection = xcb_connect(ptr::null(), ptr::null_mut());

            if connection.is_null() {
                panic!("Failed to connect to XCB");
            }

            XcbConnection {
                xcb_connection: connection,
            }
        }
    }

    pub fn get_setup(&self) -> xcb_setup_t {
        unsafe { *xcb_get_setup(self.xcb_connection) }
    }
}

impl Drop for XcbConnection {
    fn drop(&mut self) {
        unsafe {
            xcb_disconnect(self.xcb_connection);
        }
    }
}
