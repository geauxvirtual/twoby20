// Application workflow consists of reading data from an external source
// (ANT+ stick and/or Bluetooth), process the messages, and then display
// the data to the user. While the frontend is async based, the backend
// reading from USB is still synchronous. Multiple threads are used
// with data being sent between threads via channels.

// Run() is the main function to call. This handles starting up all the
// threads and configuring the channels.
pub fn run() {}
