use std::ptr;
use xcb_system::{xcb_connect, xcb_connection_t, xcb_disconnect, xcb_connection_has_error, xcb_get_setup, xcb_setup_t};

pub struct XcbConnection {
    connection: *mut xcb_connection_t,
    setup: *const xcb_setup_t
}

#[derive(Debug)]
pub enum XcbError {
    ConnectionFailed,
    UnsupportedExtension,
    InsufficientMemory,
    RequestLengthExceeded,
    DisplayParseError,
    InvalidScreen,
    UnknownError(u32)
}

impl XcbConnection {
    pub fn new() -> Result<Self, XcbError> {
        unsafe {
            let connection = xcb_connect(ptr::null(), ptr::null_mut());

            match xcb_connection_has_error(connection) as u32 {
                0 => {},
                xcb_system::XCB_CONN_ERROR => return Err(XcbError::ConnectionFailed),
                xcb_system::XCB_CONN_CLOSED_EXT_NOTSUPPORTED => return Err(XcbError::UnsupportedExtension),
                xcb_system::XCB_CONN_CLOSED_MEM_INSUFFICIENT => return Err(XcbError::InsufficientMemory),
                xcb_system::XCB_CONN_CLOSED_REQ_LEN_EXCEED => return Err(XcbError::RequestLengthExceeded),
                xcb_system::XCB_CONN_CLOSED_PARSE_ERR => return Err(XcbError::DisplayParseError),
                xcb_system::XCB_CONN_CLOSED_INVALID_SCREEN => return Err(XcbError::InvalidScreen),
                error => return Err(XcbError::UnknownError(error))
            }

            let setup = xcb_get_setup(connection);

            Ok(XcbConnection {
                connection,
                setup
            })
        }
    }

    pub fn get_setup(&self) -> xcb_setup_t {
        unsafe { *self.setup }
    }

    pub fn get_vendor(&self) -> String {
        let length = unsafe { xcb_system::xcb_setup_vendor_length(self.setup) } as usize;
        let vendor = unsafe { xcb_system::xcb_setup_vendor(self.setup) };

        let mut buf:Vec<u8> = vec![0; length+1];

        unsafe { std::ptr::copy_nonoverlapping(vendor, buf.as_mut_ptr() as *mut i8, length); };

        String::from_utf8(buf).unwrap()
    }
}

impl Drop for XcbConnection {
    fn drop(&mut self) {
        unsafe {
            xcb_disconnect(self.connection);
        }
    }
}
