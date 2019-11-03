use crate::connection::Connection;
use crate::event::EventMask;
use crate::result::XcbResult;
use std::ffi::c_void;
use xcb_system::{
    xcb_change_window_attributes, xcb_config_window_t_XCB_CONFIG_WINDOW_HEIGHT,
    xcb_config_window_t_XCB_CONFIG_WINDOW_WIDTH, xcb_config_window_t_XCB_CONFIG_WINDOW_X,
    xcb_config_window_t_XCB_CONFIG_WINDOW_Y, xcb_configure_window, xcb_cw_t_XCB_CW_EVENT_MASK,
    xcb_map_window, xcb_window_t,
};

#[derive(Debug, Copy, Clone)]
pub struct WindowHandle<'a> {
    connection: &'a Connection,
    handle: xcb_window_t,
}

impl<'a> WindowHandle<'a> {
    pub fn new(handle: u32, connection: &'a Connection) -> Self {
        Self { handle, connection }
    }

    pub fn set_event_mask(&self, events: Vec<EventMask>) -> XcbResult<()> {
        let mut mask = 0;
        for e in events {
            mask |= e as u32;
        }

        unsafe {
            let cookie = xcb_change_window_attributes(
                self.connection.get_connection(),
                self.handle,
                xcb_cw_t_XCB_CW_EVENT_MASK,
                &mask as *const u32 as *const c_void,
            );

            XcbResult::new(cookie, self.connection)
        }
    }

    pub fn map(&self) -> XcbResult<()> {
        let result = unsafe { xcb_map_window(self.connection.get_connection(), self.handle) };

        XcbResult::new(result, self.connection)
    }

    pub fn configure(&self, x: i16, y: i16, width: u16, height: u16) -> XcbResult<()> {
        let values = vec![x as u32, y as u32, u32::from(width), u32::from(height)];
        let result = unsafe {
            xcb_configure_window(
                self.connection.get_connection(),
                self.handle,
                (xcb_config_window_t_XCB_CONFIG_WINDOW_X
                    | xcb_config_window_t_XCB_CONFIG_WINDOW_Y
                    | xcb_config_window_t_XCB_CONFIG_WINDOW_WIDTH
                    | xcb_config_window_t_XCB_CONFIG_WINDOW_HEIGHT) as u16,
                values.as_ptr() as *const c_void,
            )
        };

        XcbResult::new(result, self.connection)
    }
}
