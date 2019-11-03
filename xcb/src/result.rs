use crate::connection::Connection;
use xcb_system::xcb_generic_error_t;

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

type XcbResultWithError<T> = (T, *mut xcb_generic_error_t);

pub struct XcbResult<'a, TRawReply, TReply> {
    awaiter: Box<dyn Fn(&'a Connection) -> XcbResultWithError<TRawReply>>,
    converter: Box<dyn Fn(TRawReply) -> TReply>,
    connection: &'a Connection,
}

impl<'a> XcbResult<'a, (), ()> {
    pub fn new_void(cookie: xcb_system::xcb_void_cookie_t, connection: &'a Connection) -> Self {
        Self {
            awaiter: Box::new(move |connection| {
                ((), unsafe {
                    xcb_system::xcb_request_check(connection.get_connection(), cookie)
                })
            }),
            converter: Box::new(|result| result),
            connection,
        }
    }
}

impl<'a, TRawReply, TReply> XcbResult<'a, TRawReply, TReply> {
    pub fn new(
        awaiter: Box<dyn Fn(&'a Connection) -> XcbResultWithError<TRawReply>>,
        converter: Box<dyn Fn(TRawReply) -> TReply>,
        connection: &'a Connection,
    ) -> Self {
        Self {
            awaiter,
            converter,
            connection,
        }
    }

    pub fn get_result(self) -> Result<TReply, Error> {
        let result = (self.awaiter)(self.connection);

        if result.1.is_null() {
            Ok((self.converter)(result.0))
        } else {
            Err(Error::UnknownError(
                unsafe { *(result.1) }.error_code.into(),
            ))
        }
    }
}
