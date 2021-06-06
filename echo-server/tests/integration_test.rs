use std::net::SocketAddr;
use std::net::TcpListener;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use jdn_echo_server::EchoServer;

#[test]
fn test_server_lifecycle() {
    let server_address = SocketAddr::from_str("127.0.0.1:8080").unwrap();
    // Control - assert port can be bound
    assert_port_available(server_address, "Control");
    let mut server = EchoServer::new(server_address);

    // Stop without start
    server.stop();
    assert_port_available(server_address, "Stop without start");

    // Clean start
    server.start();
    sleep_async_duration();
    assert_port_unavailable(server_address, "Clean start");

    // Double start
    server.start();
    sleep_async_duration();
    assert_port_unavailable(server_address, "Double start");

    // Clean stop
    server.stop();
    sleep_async_duration();
    assert_port_available(server_address, "Clean stop");

    // Double stop
    server.stop();
    sleep_async_duration();
    assert_port_available(server_address, "Double stop");

    // Start after external bind
    let test_server_result = TcpListener::bind(server_address);
    assert!(test_server_result.is_ok(), "External bind failed");
    let test_server = test_server_result.unwrap();
    server.start();
    sleep_async_duration();
    std::mem::drop(test_server);
    sleep_async_duration();
    assert_port_unavailable(server_address, "Start after external bind");
}

fn assert_port_available(server_address: SocketAddr, test_case: &'static str) {
    let test_server_result = TcpListener::bind(server_address);
    if let Err(e) = test_server_result {
        panic!("{} failed: {:?}", test_case, e)
    }
}

fn assert_port_unavailable(server_address: SocketAddr, test_case: &'static str) {
    let test_server_result = TcpListener::bind(server_address);
    assert!(
        test_server_result.is_err(),
        "{} failed - port is open",
        test_case
    );
    let test_server_error_kind = test_server_result.unwrap_err().kind();
    assert_eq!(
        test_server_error_kind,
        std::io::ErrorKind::AddrInUse,
        "{} failed - expected AddrInUse, got {:?}",
        test_case,
        test_server_error_kind
    );
}

fn sleep_async_duration() {
    thread::sleep(Duration::from_millis(250));
}
