systemfd --no-pid -s https::8443 -s http::8000 -- cargo watch -w src -i src/client.rs -x 'run --bin server -- -a localhost:8443'
