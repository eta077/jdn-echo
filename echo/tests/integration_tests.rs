use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use jdn_echo::EchoClient;

#[test]
fn test_client_lifecycle() {
    let server_address = SocketAddr::from_str("127.0.0.1:8080").unwrap();
    let test_server = TcpListener::bind(server_address).unwrap();
    // Control - assert client can connect
    let control_server = test_server.try_clone().unwrap();
    let control_thread = thread::spawn(move || {
        sleep_async_duration();
        assert!(control_server.accept().is_ok());
    });
    let control_client = TcpStream::connect(server_address).unwrap();
    control_thread.join().unwrap();
    std::mem::drop(control_client);

    let mut client = EchoClient::new(server_address);

    // Stop without start
    client.stop();
    assert_no_connect_attempt(&test_server, "Stop without start");

    // Clean start
    client.start();
    sleep_async_duration();
    assert_connect_successful(&test_server, "Clean start");
}

fn assert_connect_successful(test_server: &TcpListener, test_case: &'static str) {
    assert!(test_server.accept().is_ok(), "{} failed", test_case);
}

fn assert_no_connect_attempt(test_server: &TcpListener, test_case: &'static str) {
    test_server.set_nonblocking(true).unwrap();
    let result = test_server.accept();
    assert!(
        result.is_err(),
        "{} failed - connection accepted",
        test_case
    );
    println!("{:?}", result.unwrap_err());
    test_server.set_nonblocking(false).unwrap();
}

fn sleep_async_duration() {
    thread::sleep(Duration::from_millis(250));
}
