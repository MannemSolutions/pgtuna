//! This is a sample named pipe client.
//! The client expects a named pipe to be created and provided as a command line
//! argument.
//! The client opens the named pipe for writing and emits randomly generated numbers
//! into the pipe, separated by newlines.

extern crate rand;
extern crate unix_named_pipe;

use rand::prelude::*;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::{env, thread, time};
use unix_named_pipe::FileFIFOExt;

fn main() {
    let pipe_path = env::args()
        .nth(1)
        .expect("named pipe path required but not provided");
    println!("client opening pipe: {}", pipe_path);

    let mut pipe = try_open(&pipe_path).expect("could not open pipe for writing");

    loop {
        let payload = [random::<u8>(), 0x0a];
        println!("sending number: {}", payload[0]);

        let res = pipe
            .write(&payload)
            .expect("could not write payload to pipe");
        if res != payload.len() {
            println!("could not write 2 bytes to pipe");
            break;
        }

        // Not necessary, but sleep a short period of time before writing more numbers
        // to the pipe
        thread::sleep(time::Duration::from_millis(500));
    }
}

/// Tries to open the pipe at `pipe_path`.
///   1. Attempt to open the path for writing
///     a. If `open_write()` fails with `io::ErrorKind::NotFound`, create the pipe and try again
///     b. If `open_write()` fails with any other error, raise the error.
///   2. Now that the file is opened for writing, ensure that it is a named pipe
///     a. If `is_fifo()` fails, panic.
///     b. If `is_fifo()` returns `false`, panic.
///   3. Return the newly opened pipe file wrapped in an `io::Result`
fn try_open<P: AsRef<Path> + Clone>(pipe_path: P) -> io::Result<fs::File> {
    let pipe = unix_named_pipe::open_write(&pipe_path);
    if let Err(err) = pipe {
        match err.kind() {
            io::ErrorKind::NotFound => {
                println!("creating pipe at: {:?}", pipe_path.clone().as_ref());
                unix_named_pipe::create(&pipe_path, Some(0o660))?;
                println!("created pipe at: {:?}", pipe_path.clone().as_ref());

                // Note that this has the possibility to recurse forever if creation `open_write`
                // fails repeatedly with `io::ErrorKind::NotFound`, which is certainly not nice behaviour.
                return try_open(pipe_path);
            }
            _ => {
                println!("This is it:");
                return Err(err);
            }
        }
    }
    println!("opened pipe at: {:?}", pipe_path.clone().as_ref());

    let pipe_file = pipe.unwrap();
    let is_fifo = pipe_file
        .is_fifo()
        .expect("could not read type of file at pipe path");
    if !is_fifo {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "expected file at {:?} to be fifo, is actually {:?}",
                &pipe_path.clone().as_ref(),
                pipe_file.metadata()?.file_type(),
            ),
        ));
    }

    Ok(pipe_file)
}