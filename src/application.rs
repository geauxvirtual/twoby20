// Application workflow consists of reading data from an external source
// (ANT+ stick and/or Bluetooth), process the messages, and then display
// the data to the user. While the frontend is async based, the backend
// reading from USB is still synchronous. Multiple threads are used
// with data being sent between threads via channels.
use std::thread;

use libant::{Ant, Context};

// Run() is the main function to call. This handles starting up all the
// threads and configuring the channels.
pub fn run() {
    let (_ant_request_tx, ant_request_rx) = libant::unbounded();
    let (ant_message_tx, _ant_message_rx) = libant::unbounded();

    let mut context = Context::new().unwrap();
    let mut ant = Ant::init(&mut context, ant_request_rx, ant_message_tx).unwrap();

    let ant_run_handle = thread::spawn(move || {
        let _ = ant.run();
    });

    ant_run_handle.join().unwrap();
}
