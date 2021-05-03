# nc-vhost
A trivial implementation of `nc` command with virtual host.

Only client -> server direction is supported.

```
# Lauch server
$ cargo run --bin front-server
$ cargo run --bin back-server 127.0.0.1:10001
$ cargo run --bin back-server 127.0.0.1:10002

# connect servers from client
$ cargo run --bin client -- 127.0.0.1:8080 service1
$ cargo run --bin client -- 127.0.0.1:8080 service2
```