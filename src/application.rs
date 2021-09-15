// Application workflow consists of reading data from an external source
// (ANT+ stick and/or Bluetooth), process the messages, and then display
// the data to the user. While the frontend is async based, the backend
// reading from USB is still synchronous. Multiple threads are used
// with data being sent between threads via channels.
use std::thread;

// Run() is the main function to call. This handles starting up all the
// threads and configuring the channels.
pub fn run() {
    let (_ant_request_tx, ant_request_rx) = libant::unbounded();
    let (ant_message_tx, ant_message_rx) = libant::unbounded();

    let ant_run_handle = thread::spawn(move || libant::ant::run(ant_request_rx, ant_message_tx));

    loop {
        match ant_message_rx.recv() {
            Ok(libant::Response::Error(e)) => {
                log::error!("Error message received: {:?}", e);
            }
            Ok(mesg) => log::debug!("Debugging message received: {:?}", mesg),
            Err(e) => {
                log::error!("Error receiving from Ant run thread: {:?}", e);
                break;
            }
        }
    }
    ant_run_handle.join().unwrap();
}
