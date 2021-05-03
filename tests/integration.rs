use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use tokio::process::Command;
use std::process::Stdio;
use std::env;
use tokio::join;

#[tokio::test]
async fn read() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_front-server"))
        .kill_on_drop(true)
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start front-server");
    let stdout = child.stdout.take().unwrap();
    let mut front_reader = BufReader::new(stdout).lines();

    let mut child = Command::new(env!("CARGO_BIN_EXE_back-server"))
        .kill_on_drop(true)
        .arg("127.0.0.1:10001")
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start back-server");
    let stdout = child.stdout.take().unwrap();
    let mut back_reader = BufReader::new(stdout).lines();

    let mut child = Command::new(env!("CARGO_BIN_EXE_client"))
        .kill_on_drop(true)
        .arg("127.0.0.1:8080")
        .arg("service1")
        .stdin(Stdio::piped())
        .spawn()
        .expect("failed to start client");
    let mut stdin = child.stdin.take().unwrap();
    let write = async move { stdin.write_all("foobarbaz\n".as_bytes()).await.expect("write stdin") };

    let data = front_reader
        .next_line()
        .await
        .unwrap_or_else(|_| Some(String::new()))
        .expect("failed to read line");
    assert_eq!(data, "Listening on: 127.0.0.1:8080");

    let data = back_reader
        .next_line()
        .await
        .unwrap_or_else(|_| Some(String::new()))
        .expect("failed to read line");
    assert_eq!(data, "Listening on: 127.0.0.1:10001");

    let front_read = async move {
        front_reader
            .next_line()
            .await
            .unwrap_or_else(|_| Some(String::new()))
            .expect("failed to read line")
    };

    let back_read = async move {
        back_reader
            .next_line()
            .await
            .unwrap_or_else(|_| Some(String::new()))
            .expect("failed to read line")
    };

    let (_write, front_read, back_read) =
        join!(write, front_read, back_read);

    assert_eq!(front_read, "[service1]: foobarbaz");
    assert_eq!(back_read, "foobarbaz");
}