
// src/bin/server.rs
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

use anyhow::Context;

use pgtuna::{
    socket::SocketBuilder,
};

fn main() -> anyhow::Result<()> {
    let socket_path = "/tmp/.s.pgtuna";
    // Create the socket
    // Have a look at socket.rs to see what this does
    let socket = SocketBuilder::new()
        .with_path(socket_path)
        .with_permissions(0o700)
        .nonblocking(false)
        .build()
        .context("Could not create the socket")?;

    println!("Starting the unix socket server, Press Ctrl^C to stop...");

    // the loop allows to handle several connections, one after the other
    loop {
        // accept_connection() is a wrapper around UnixListener::accept(), check socket.rs
        let (unix_stream, socket_address) = socket.accept_connection()?;

        println!(
            "Accepted connection. Stream: {:?}, address: {:?}",
            unix_stream, socket_address
        );

        handle_connection(unix_stream)?;
    }
}

fn handle_connection(mut stream: UnixStream) -> anyhow::Result<()> {
    // receive a message using normal read logic
    let mut message = String::new();
    stream
        .read_to_string(&mut message)
        .context("Failed at reading the unix stream")?;

    println!("{}", message);

    let body =
        "work_mem = 64MB"
        .as_bytes();

    // pretty normal write logic
    println!("Sending body");
    stream
        .write(body)
        .context("Could not write processing response onto the unix stream")?;

    Ok(())
}