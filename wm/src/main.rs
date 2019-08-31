use xcb::Event;

fn main() {
    let connection = xcb::XcbConnection::new().unwrap();
    let root_window = connection.get_root_window();
    root_window.set_event_mask(vec![Event::SubstructureNotify, Event::SubstructureRedirect]);

    println!("Vendor: {}", connection.get_vendor());
    println!("Window: {:?}", root_window);
}
