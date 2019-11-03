use xcb::{EventMask, XcbEvent};

fn main() {
    let connection = xcb::XcbConnection::new().unwrap();
    let root_window = connection.get_root_window();
    root_window
        .set_event_mask(vec![EventMask::SubstructureNotify, EventMask::SubstructureRedirect])
        .get_result()
        .expect("Failed to get SubstructureNotify and SubstructureRedirect event masks. Is another WM already running?");

    println!("Vendor: {}", connection.get_vendor());
    println!("Window: {:?}", root_window);

    loop {
        let event = connection.wait_for_event();

        match event {
            XcbEvent::WindowConfigurationRequest {
                window,
                x,
                y,
                width,
                height,
            } => window
                .configure(x, y, width, height)
                .get_result()
                .expect("Failed to configure window"),
            XcbEvent::WindowMappingRequest { window } => {
                window.map().get_result().expect("Failed to map window")
            }
            e => println!("[ ] Got an event! {:?}", e),
        }
    }
}
