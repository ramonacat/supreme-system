use xcb::event::{Event, EventMask};
use xcb::window::{OwnedWindow, Window};
use xcb::Rectangle;

fn main() {
    let connection = xcb::connection::Connection::new().unwrap();
    let root_window = connection.get_root_window();
    root_window
        .set_event_mask(vec![EventMask::SubstructureNotify, EventMask::SubstructureRedirect])
        .get_result()
        .expect("Failed to get SubstructureNotify and SubstructureRedirect event masks. Is another WM already running?");

    println!("Vendor: {}", connection.get_vendor());
    println!("Window: {:?}", root_window);

    let mut windows = vec![];

    loop {
        let event = connection.wait_for_event();

        match event {
            Event::WindowConfigurationRequest { window, rectangle } => window
                .configure(rectangle)
                .get_result()
                .expect("Failed to configure window"),
            Event::WindowMappingRequest { window } => {
                let geometry = window
                    .get_geometry()
                    .get_result()
                    .expect("Failed to get window geometry");

                let new_parent = OwnedWindow::new(
                    &connection,
                    Rectangle {
                        x: geometry.rectangle.x,
                        y: geometry.rectangle.y,
                        width: geometry.rectangle.width,
                        height: geometry.rectangle.height + 30,
                    },
                );

                new_parent.map().get_result().expect("Failed to map window");

                new_parent
                    .set_event_mask(vec![
                        EventMask::SubstructureNotify,
                        EventMask::SubstructureRedirect,
                    ])
                    .get_result()
                    .expect("Failed to set event mask during reparenting");

                window.reparent(&new_parent, 0, 30);

                window.map().get_result().expect("Failed to map window");

                windows.push((new_parent, window));
            }
            Event::WindowUnmapped { window } => {
                let parent = &windows.iter().find(|(_, child)| child.id() == window.id());

                if let Some(parent_child) = parent {
                    parent_child.0.unmap();
                }
            }
            Event::WindowDestroyed { window } => {
                windows.retain(|(_, child)| child.id() != window.id());
            }
            e => println!("[ ] Got an event! {:?}", e),
        }
    }
}
