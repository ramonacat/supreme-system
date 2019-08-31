fn main() {
    let connection = xcb::XcbConnection::new().unwrap();
    let root_window = connection.get_root_window();
    println!("Vendor: {}", connection.get_vendor());
    println!("Window: {:?}", root_window);
}
