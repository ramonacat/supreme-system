use crate::connection::Connection;
use crate::event::EventMask;
use crate::result::XcbResult;
use crate::Rectangle;
use std::ffi::c_void;
use xcb_system::{
    xcb_change_window_attributes_checked, xcb_config_window_t_XCB_CONFIG_WINDOW_HEIGHT,
    xcb_config_window_t_XCB_CONFIG_WINDOW_WIDTH, xcb_config_window_t_XCB_CONFIG_WINDOW_X,
    xcb_config_window_t_XCB_CONFIG_WINDOW_Y, xcb_configure_window, xcb_cw_t_XCB_CW_EVENT_MASK,
    xcb_generic_error_t, xcb_get_geometry_reply_t, xcb_get_window_attributes_reply_t,
    xcb_map_window, xcb_unmap_window, xcb_window_t,
};

#[derive(Debug, Copy, Clone)]
pub struct WindowHandle<'a> {
    connection: &'a Connection,
    handle: xcb_window_t,
}

#[derive(Debug, Copy, Clone)]
// todo rename to just `Attributes`, add fields
pub struct WindowAttributes {}

#[derive(Debug, Copy, Clone)]
pub struct Geometry {
    pub rectangle: Rectangle,
}

pub trait Window {
    fn set_event_mask(&self, events: Vec<EventMask>) -> XcbResult<(), ()>;
    fn map(&self) -> XcbResult<(), ()>;
    fn unmap(&self) -> XcbResult<(), ()>;
    fn configure(&self, rectangle: Rectangle) -> XcbResult<(), ()>;
    fn get_attributes(&self) -> XcbResult<xcb_get_window_attributes_reply_t, WindowAttributes>;
    fn get_geometry(&self) -> XcbResult<xcb_get_geometry_reply_t, Geometry>;

    fn reparent(&self, new_parent: &dyn Window, x_offset: i16, y_offset: i16) -> XcbResult<(), ()>;
    fn id(&self) -> u32;
}

pub struct OwnedWindow<'a> {
    handle: WindowHandle<'a>,
}

impl<'a> OwnedWindow<'a> {
    pub fn new(connection: &'a Connection, rectangle: Rectangle) -> Self {
        let handle = unsafe { xcb_system::xcb_generate_id(connection.get_connection()) };
        unsafe {
            xcb_system::xcb_create_window(
                connection.get_connection(),
                xcb_system::XCB_COPY_FROM_PARENT as u8,
                handle,
                connection.get_root_window().handle,
                rectangle.x,
                rectangle.y,
                rectangle.width,
                rectangle.height,
                0,
                xcb_system::xcb_window_class_t_XCB_WINDOW_CLASS_INPUT_OUTPUT as u16,
                connection.get_root_visual(),
                0,
                std::ptr::null(),
            )
        };

        Self {
            handle: WindowHandle::new(handle, connection),
        }
    }
}

impl Window for OwnedWindow<'_> {
    fn set_event_mask(&self, events: Vec<EventMask>) -> XcbResult<(), ()> {
        self.handle.set_event_mask(events)
    }

    fn map(&self) -> XcbResult<(), ()> {
        self.handle.map()
    }

    fn unmap(&self) -> XcbResult<(), ()> {
        self.handle.unmap()
    }

    fn configure(&self, rectangle: Rectangle) -> XcbResult<(), ()> {
        self.handle.configure(rectangle)
    }

    fn get_attributes(&self) -> XcbResult<xcb_get_window_attributes_reply_t, WindowAttributes> {
        self.handle.get_attributes()
    }

    fn get_geometry(&self) -> XcbResult<xcb_get_geometry_reply_t, Geometry> {
        self.handle.get_geometry()
    }

    fn reparent(&self, new_parent: &dyn Window, x_offset: i16, y_offset: i16) -> XcbResult<(), ()> {
        self.handle.reparent(new_parent, x_offset, y_offset)
    }

    fn id(&self) -> u32 {
        self.handle.id()
    }
}

impl Drop for OwnedWindow<'_> {
    fn drop(&mut self) {
        self.handle
            .destroy()
            .get_result()
            .expect("Failed to destroy window");
    }
}

impl<'a> WindowHandle<'a> {
    pub fn new(handle: u32, connection: &'a Connection) -> Self {
        Self { handle, connection }
    }

    pub fn destroy(&self) -> XcbResult<(), ()> {
        let cookie = unsafe {
            xcb_system::xcb_destroy_window(self.connection.get_connection(), self.handle)
        };

        XcbResult::new_void(cookie, self.connection)
    }
}

impl Window for WindowHandle<'_> {
    fn set_event_mask(&self, events: Vec<EventMask>) -> XcbResult<(), ()> {
        let mut mask = 0;
        for e in events {
            mask |= e as u32;
        }

        unsafe {
            let cookie = xcb_change_window_attributes_checked(
                self.connection.get_connection(),
                self.handle,
                xcb_cw_t_XCB_CW_EVENT_MASK,
                &mask as *const u32 as *const c_void,
            );

            XcbResult::new_void(cookie, self.connection)
        }
    }

    fn map(&self) -> XcbResult<(), ()> {
        let result = unsafe { xcb_map_window(self.connection.get_connection(), self.handle) };

        XcbResult::new_void(result, self.connection)
    }

    fn unmap(&self) -> XcbResult<(), ()> {
        let result = unsafe { xcb_unmap_window(self.connection.get_connection(), self.handle) };

        XcbResult::new_void(result, self.connection)
    }

    fn configure(&self, rectangle: Rectangle) -> XcbResult<(), ()> {
        let values = vec![
            rectangle.x as u32,
            rectangle.y as u32,
            u32::from(rectangle.width),
            u32::from(rectangle.height),
        ];
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

        XcbResult::new_void(result, self.connection)
    }

    fn get_attributes(&self) -> XcbResult<xcb_get_window_attributes_reply_t, WindowAttributes> {
        let cookie = unsafe {
            xcb_system::xcb_get_window_attributes(self.connection.get_connection(), self.handle)
        };

        XcbResult::new(
            Box::new(move |connection| {
                let mut error: *mut xcb_generic_error_t = std::ptr::null_mut();

                let reply = unsafe {
                    *xcb_system::xcb_get_window_attributes_reply(
                        connection.get_connection(),
                        cookie,
                        &mut error,
                    )
                };

                (reply, error)
            }),
            Box::new(|_reply| WindowAttributes {}),
            self.connection,
        )
    }

    fn get_geometry(&self) -> XcbResult<xcb_get_geometry_reply_t, Geometry> {
        let cookie =
            unsafe { xcb_system::xcb_get_geometry(self.connection.get_connection(), self.handle) };

        XcbResult::new(
            Box::new(move |connection| {
                let mut error: *mut xcb_generic_error_t = std::ptr::null_mut();

                let reply = unsafe {
                    *xcb_system::xcb_get_geometry_reply(
                        connection.get_connection(),
                        cookie,
                        &mut error,
                    )
                };

                (reply, error)
            }),
            Box::new(|reply| Geometry {
                rectangle: Rectangle {
                    x: reply.x,
                    y: reply.y,
                    width: reply.width,
                    height: reply.height,
                },
            }),
            self.connection,
        )
    }

    fn reparent(&self, new_parent: &dyn Window, x_offset: i16, y_offset: i16) -> XcbResult<(), ()> {
        let cookie = unsafe {
            xcb_system::xcb_reparent_window(
                self.connection.get_connection(),
                self.handle,
                new_parent.id(),
                x_offset,
                y_offset,
            )
        };

        XcbResult::new_void(cookie, self.connection)
    }

    fn id(&self) -> u32 {
        self.handle
    }
}
