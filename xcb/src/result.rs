use crate::connection::Connection;

#[derive(Debug)]
pub enum Error {
    ConnectionFailed,
    UnsupportedExtension,
    InsufficientMemory,
    RequestLengthExceeded,
    DisplayParseError,
    InvalidScreen,
    UnknownError(u32),
    ScreenNotFound(u32),
}

pub struct XcbResult<'a, T> {
    cookie: xcb_system::xcb_void_cookie_t,
    connection: &'a Connection,
    _marker: std::marker::PhantomData<T>,
}

impl<'a> XcbResult<'a, ()> {
    pub fn new(cookie: xcb_system::xcb_void_cookie_t, connection: &'a Connection) -> Self {
        Self {
            cookie,
            connection,
            _marker: std::marker::PhantomData {},
        }
    }

    pub fn get_result(&mut self) -> Result<(), Error> {
        let result =
            unsafe { xcb_system::xcb_request_check(self.connection.get_connection(), self.cookie) };

        if result.is_null() {
            Ok(())
        } else {
            Err(Error::UnknownError(unsafe { *result }.error_code.into()))
        }
    }
}
