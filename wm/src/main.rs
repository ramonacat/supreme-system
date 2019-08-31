fn main() {
    let connection = xcb::XcbConnection::new().unwrap();
    println!("Vendor: {}", connection.get_vendor())
}
