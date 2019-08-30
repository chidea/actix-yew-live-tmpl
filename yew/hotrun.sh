cargo watch -w static -w src -s 'yarn build && ../target/release/client -u https://localhost:8443/ws -m restart_all'
