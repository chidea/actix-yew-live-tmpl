# actix-yew-live-tmpl
Hot reloading local developing server template based on [Actix](https://actix.rs) and [Yew](https://github.com/yewstack/yew) with [Redis](https://redis.io/) and [Rustls](https://github.com/ctz/rustls).
Naturally, it's an wasm frontend on a TLS activated HTTPS and HTTP/2 web server.
Temped to be a kickstarting point for various web project requiring bleeding-edge level of technologies.

### Prerequisites
- npm
  - but the stack does not run on nodejs. It is replaced with Actix instead.
  - [wasm-pack-npm](https://www.npmjs.com/package/wasm-pack-npm)
    - Installs [wasm-pack](https://www.npmjs.com/package/wasm-pack-npm) and wasm32-unknown-unknown target
    - `sudo npm i -g wasm-pack-npm`
- [cargo-web](https://crates.io/crates/cargo-web)
  - Required to compile yew for Actix.
  - `cargo install cargo-web`
- [cargo-generate](https://crates.io/crates/cargo-generate)
  - Clones this repository and rename it as your wanted project name.
  - `cargo install cargo-generate`
  - Requires openssl, pkg-config
- [redis](https://redis.io)
  - DB

### How to initialize
```
cargo generate --git https://github.com/chidea/actix-yew-live-tmpl.git
# give project name
cd <project name>/yew
npm i
cd ..
./client.sh # builds actix websocket client in release mode
```

### How to start hot-loading
After initializing, open 2 terminals and start `hotrun.sh` scripts in each front/back end directories.
Redis server (`redis-server`) must be running before.

```
cd <project name>/yew
./hotrun.sh
```

```
cd <project name>
./hotrun.sh
```

Open `http://localhost:8000` or `https://localhost:8443` with your browser.
Now, whenever you edit front/back end codes, newly compiled version of the stack kicks off over socket swapping and the browser automatically reloads on it.

### How to test DB (Redis)
Go to `https://localhost:8443/db`

### Limitation
- When there are some compile error on front-end (Yew) side, it may break hot-loading thus it may be required to press F5 manually after fixing it.
- WebSocket client is bypassing server ssl checking.

### Potential upgrades
- Static template engine with [askama](https://crates.io/crates/askama)

### Generalized Websocket Client
Attached bin named `client` is a generalized websocket client.
It is currently not checking server side TLS certificate but in this way, it supports https(wss) connection.
```
$ target/release/client -h
actix-yew-live-tmpl 0.1.0
Generalized WebSocket Client

USAGE:
 -m <msg>        Message to send. Set it to '-' to read stdin to send, leave it blank to use stdin as console loop to
                 send multiple messages. [default: ]
 -u <url>        Address to connect [default: https://localhost:443/ws]
```
