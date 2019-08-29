fn main() {
    let connection = xcb::XcbConnection::new();
    let setup = connection.get_setup();

    println!(
        "X11 Protocol version: {}.{}",
        setup.protocol_major_version,
        setup.protocol_minor_version
    );
}
