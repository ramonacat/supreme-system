use xcb::event::{Event, EventMask, MouseButton};
use xcb::window::{OwnedWindow, Window};
use xcb::Rectangle;

fn main() {
    let connection = xcb::connection::Connection::new().unwrap();
    let root_window = connection.get_root_window().unwrap();
    root_window
        .set_event_mask(vec![
            EventMask::SubstructureNotify,
            EventMask::SubstructureRedirect,
            EventMask::ButtonPress,
            EventMask::ButtonRelease
        ])
        .get_result()
        .expect("Failed to get SubstructureNotify and SubstructureRedirect event masks. Is another WM already running?");

    println!("Vendor: {}", connection.get_vendor().unwrap());
    println!("Window: {:?}", root_window);

    let mut windows = vec![];
    let mut move_start = None;
    let mut move_window = None;

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
                )
                .unwrap();

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
            Event::ButtonPressed {
                button: MouseButton::Left,
                child_window,
                ..
            } => {
                let grabbed = connection.grab_pointer().get_result();
                // todo replace with actual error handling & logging
                println!("Grabbed? {:?}", grabbed);
                move_window = Some(child_window.unwrap().id());
            }
            Event::ButtonReleased {
                button: MouseButton::Left,
                ..
            } => {
                connection.ungrab_pointer().get_result().unwrap(); // todo only do that if we actually grabbed the pointer previously
                move_start = None;
                move_window = None;
            }
            Event::MotionNotify { x, y, window } => {
                println!("Motion notify: {:?}", window);
                println!("Windows.first: {:?}", windows.first());

                if let Some((start_x, start_y)) = move_start {
                    let offset = (x - start_x, y - start_y);
                    let parent = &windows
                        .iter()
                        .find(|(parent, _)| parent.id() == move_window.unwrap())
                        .unwrap()
                        .0;

                    let current_geometry = parent.get_geometry().get_result().unwrap();
                    parent
                        .configure(Rectangle {
                            x: current_geometry.rectangle.x + offset.0,
                            y: current_geometry.rectangle.y + offset.1,
                            width: current_geometry.rectangle.width,
                            height: current_geometry.rectangle.height,
                        })
                        .get_result()
                        .unwrap();
                }

                move_start = Some((x, y));
            }
            e => println!("[ ] Got an event! {:?}", e),
        }
    }
}
