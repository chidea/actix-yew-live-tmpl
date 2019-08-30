// #[macro_use]
extern crate actix;
// extern crate byteorder;
// extern crate bytes;
extern crate futures;
extern crate serde;
extern crate serde_json;
// extern crate tokio_io;
// extern crate tokio_tcp;
extern crate awc;
extern crate rustls;
extern crate structopt;
#[macro_use]
extern crate log;
extern crate env_logger;

// #[macro_use]
extern crate serde_derive;

use actix::{
    // prelude::*, io::FramedWrite
    io::{SinkWrite, WriteHandler},
    prelude::*,
    Actor,
    ActorContext,
    AsyncContext,
    Context,
    Handler,
    StreamHandler,
};
use actix_codec::{AsyncRead, AsyncWrite, Framed};
use futures::{
    lazy,
    /* future::ok,  */ stream::{SplitSink, Stream},
    Future,
};
use std::{
    io,
    // str::FromStr,
    // time::Duration,
    sync::Arc,
    thread,
    // net, process, thread,
};
// use tokio_io::{AsyncRead, io::WriteHalf};
// use tokio_tcp::TcpStream;
use awc::{
    error::WsProtocolError,
    http::StatusCode,
    ws::{Codec, Frame, Message},
    Client, Connector,
};
use rustls::ClientConfig;
use structopt::StructOpt;

// use webpki;
// use webpki_roots;

// mod codec;

mod server;
mod ws;
// mod util;
use ws::HEARTBEAT_INTERVAL;

#[derive(StructOpt, Debug, Clone)]
/// VIS Server
pub struct Opt {
    /// Address to connect
    #[structopt(short = "u", default_value = "https://localhost:443/ws")]
    url: String,
    /// Message to send. Set it to '-' to read stdin to send,
    /// leave it blank to use stdin as console loop to send multiple messages.
    #[structopt(short = "m", default_value = "")]
    msg: String,
}

mod danger {
    use rustls::{
        self, Certificate, RootCertStore, ServerCertVerified, ServerCertVerifier, TLSError,
    };
    use webpki;

    pub struct NoCertificateVerification {}

    impl ServerCertVerifier for NoCertificateVerification {
        fn verify_server_cert(
            &self,
            _roots: &RootCertStore,
            _presented_certs: &[Certificate],
            _dns_name: webpki::DNSNameRef<'_>,
            _ocsp: &[u8],
        ) -> Result<ServerCertVerified, TLSError> {
            Ok(ServerCertVerified::assertion())
        }
    }
}

fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let opt = Opt::from_args();

    // let sys = System::new("ws-client");
    System::run(move || {
        let mut cfg = ClientConfig::new();
        // let protos = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        // cfg.set_protocols(&protos);
        cfg.dangerous()
            .set_certificate_verifier(Arc::new(danger::NoCertificateVerification {}));

        let client = Client::build()
            .connector(Connector::new().rustls(Arc::new(cfg)).finish())
            .finish();

        // sys.block_on(
        Arbiter::spawn(lazy(move || {
            client
                .ws(&opt.url)
                .connect()
                .map_err(|e| panic!("{}", e))
                .map(move |(response, framed)| {
                    let sys = System::current();
                    if response.status() != StatusCode::SWITCHING_PROTOCOLS {
                        sys.stop();
                    }

                    let (sink, stream) = framed.split();
                    let addr = WsClient::create(|ctx| {
                        WsClient::add_stream(stream, ctx);
                        WsClient(SinkWrite::new(sink, ctx))
                    });

                    let read_stdin = || -> String {
                        let mut cmd = String::new();
                        if io::stdin().read_line(&mut cmd).is_err() {
                            println!("error");
                        }
                        cmd
                    };

                    if opt.msg.is_empty() {
                        // start console loop
                        thread::spawn(move || loop {
                            addr.do_send(ClientCommand(read_stdin()));
                        });
                    } else if opt.msg == "-" {
                        addr.do_send(ClientCommand(read_stdin()));
                        sys.stop();
                    } else {
                        addr.do_send(ClientCommand(opt.msg));
                        sys.stop();
                    }
                })
        }));
    })

    // ).unwrap();

    // sys.block_on(

    // ).unwrap();
    // Arbiter::spawn(
    //   TcpStream::connect(&addr)
    //       .and_then(|stream| {
    //         let addr = WsClient::create(|ctx| {
    //           let (r, w) = stream.split();
    //           WsClient::add_stream(
    //             FramedRead::new(r, codec::ClientWsCodec),
    //             ctx,
    //           );
    //           WsClient {
    //             framed: FramedWrite::new(
    //               w,
    //               codec::ClientWsCodec,
    //               ctx,
    //             ),
    //           }
    //         });

    //         // start console loop
    //         thread::spawn(move || loop {
    //           let mut cmd = String::new();
    //           if io::stdin().read_line(&mut cmd).is_err() {
    //             println!("error");
    //             return;
    //           }

    //           addr.do_send(ClientCommand(cmd));
    //         });

    //         ok(())
    //       })
    //       .map_err(|e| {
    //         println!("Can not connect to server: {}", e);
    //         process::exit(1)
    //       }),
    // );

    // println!("Running ws client");
    // sys.run()
}

// struct WsClient {
//   framed: FramedWrite<WriteHalf<TcpStream>, codec::ClientWsCodec>,
// }

// #[derive(Message)]
// struct ClientCommand(String);

// impl Actor for WsClient {
//   type Context = Context<Self>;

//   fn started(&mut self, ctx: &mut Context<Self>) {
//     // start heartbeats otherwise server will disconnect after 10 seconds
//     self.hb(ctx)
//   }

//   fn stopped(&mut self, _: &mut Context<Self>) {
//     println!("Disconnected");

//     // Stop application on disconnect
//     System::current().stop();
//   }
// }

// impl WsClient {
//   fn hb(&self, ctx: &mut Context<Self>) {
//     ctx.run_later(Duration::new(, 0), |act, ctx| {
//       act.framed.write(codec::WsRequest::Ping);
//       act.hb(ctx);

//       // client should also check for a timeout here, similar to the
//       // server code
//     });
//   }
// }

// impl actix::io::WriteHandler<io::Error> for WsClient {}

// /// Handle stdin commands
// impl Handler<ClientCommand> for WsClient {
//   type Result = ();

//   fn handle(&mut self, msg: ClientCommand, _: &mut Context<Self>) {
//     let m = msg.0.trim();
//     if m.is_empty() {
//       return;
//     }

//     // we check for /sss type of messages
//     // if m.starts_with('/') {
//     //   let v: Vec<&str> = m.splitn(2, ' ').collect();
//     //   match v[0] {
//     //     "/list" => {
//     //       self.framed.write(codec::WsRequest::List);
//     //     }
//     //     "/join" => {
//     //       if v.len() == 2 {
//     //         self.framed.write(codec::WsRequest::Join(v[1].to_owned()));
//     //       } else {
//     //         println!("!!! room name is required");
//     //       }
//     //     }
//     //     _ => println!("!!! unknown command"),
//     //   }
//     // } else {
//       self.framed.write(codec::WsRequest::Message(m.to_owned()));
//     // }
//   }
// }

// /// Server communication
// impl StreamHandler<codec::WsResponse, io::Error> for WsClient {
//   fn handle(&mut self, msg: codec::WsResponse, _: &mut Context<Self>) {
//     match msg {
//       codec::WsResponse::Message(ref msg) => {
//         println!("message: {}", msg);
//       }
//       // codec::WsResponse::Joined(ref msg) => {
//       //   println!("!!! joined: {}", msg);
//       // }
//       // codec::WsResponse::Rooms(rooms) => {
//       //   println!("\n!!! Available rooms:");
//       //   for room in rooms {
//       //     println!("{}", room);
//       //   }
//       //   println!("");
//       // }
//       _ => (),
//     }
//   }
// }

struct WsClient<T>(SinkWrite<SplitSink<Framed<T, Codec>>>)
where
    T: AsyncRead + AsyncWrite;

#[derive(Message)]
struct ClientCommand(String);

impl<T: 'static> Actor for WsClient<T>
where
    T: AsyncRead + AsyncWrite,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopped(&mut self, _: &mut Context<Self>) {
        info!("Disconnected");

        // Stop application on disconnect
        System::current().stop();
    }
}

impl<T: 'static> WsClient<T>
where
    T: AsyncRead + AsyncWrite,
{
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(HEARTBEAT_INTERVAL, |act, ctx| {
            act.0.write(Message::Ping(String::new())).unwrap();
            act.hb(ctx);

            // client should also check for a timeout here, similar to the
            // server code
        });
    }
}

/// Handle stdin commands
impl<T: 'static> Handler<ClientCommand> for WsClient<T>
where
    T: AsyncRead + AsyncWrite,
{
    type Result = ();

    fn handle(&mut self, msg: ClientCommand, _ctx: &mut Context<Self>) {
        self.0.write(Message::Text(msg.0)).unwrap();
    }
}

/// Handle server websocket messages
impl<T: 'static> StreamHandler<Frame, WsProtocolError> for WsClient<T>
where
    T: AsyncRead + AsyncWrite,
{
    fn handle(&mut self, msg: Frame, _ctx: &mut Context<Self>) {
        match msg {
            Frame::Text(txt) => println!("Server: {:?}", txt),
            _ => (),
        }
    }

    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("Connected");
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        info!("Server disconnected");
        ctx.stop()
    }
}

impl<T: 'static> WriteHandler<WsProtocolError> for WsClient<T> where T: AsyncRead + AsyncWrite {}
