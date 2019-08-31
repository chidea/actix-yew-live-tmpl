extern crate actix_codec;
extern crate actix_rt;
// extern crate actix_server;
extern crate actix_http;
extern crate actix_service;
// extern crate actix_connect;
extern crate actix_files;
extern crate actix_web;
extern crate actix_web_actors;

extern crate futures;
extern crate listenfd;
extern crate openssl;
extern crate rustls;
extern crate structopt;
extern crate webpki;
extern crate webpki_roots;
#[macro_use]
extern crate redis_async;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate askama;
extern crate env_logger;
// #[macro_use]
// extern crate lazy_static;

mod server;
mod tls;
// mod util;
// mod model;
mod db;
mod ws;
// mod codec;
// mod session;

use std::{
    env,
    // fmt,
    io,
    // sync::{
    // atomic::{AtomicUsize, Ordering},
    // Arc,
    // },
    // net::{ToSocketAddrs},
};

use actix::*;
// use actix_codec::{AsyncRead, AsyncWrite};
use actix_rt::System;
// use actix_http::{h1, h2, ws, Response, ServiceConfig, HttpService, Request};
use actix_web::{http::header, middleware, web, App, HttpResponse, HttpServer};
// use actix_server::{Io, Server};
// use actix_service::{
// NewService,
// service_fn,
// IntoNewService,
// };
use actix_files::Files;
use actix_redis::{RedisActor, RedisSession};
// use actix_server_config::ServerConfig;
// use futures::{
// future::{ok},
// Future
// };
use db::db_route;
use listenfd::ListenFd;
use server::WsServer;
use structopt::StructOpt;
use ws::ws_route;

// use tokio_rustls::{TlsConnector, TlsAcceptor, TlsStream};
// use tokio_tcp::{TcpStream};

#[derive(StructOpt, Debug, Clone)]
/// Actix-Yew-live-tmpl webserver launcher
pub struct Opt {
    /// Debug actix-net
    #[structopt(short = "v")]
    debug: bool,

    /// Address to listen or bind with tls
    #[structopt(short = "a", default_value = "localhost:443")]
    addr: String,

    /// http port to redirect to the tls service
    #[structopt(short = "r", default_value = "80")]
    rport: u16,

    // /// websocket port
    // #[structopt(default_value = "12345")]
    // tcpport: u16,
    #[structopt(short = "q", long = "reqauth")]
    flag_require_auth: bool,

    /// Enable client authentication, and accept certificates signed by those roots provided in <auth>. Leave it unset to use Mozilla root certs and set it "" to disable client authentication.
    #[structopt(short = "t", long = "auth")]
    flag_auth: Option<String>,

    /// Read server certificates from <certs>. This should contain PEM-format certificates in the right order (the first certificate should certify KEYFILE, the last should be a root CA).
    #[structopt(short = "c", long = "certs", default_value = "cert.pem")]
    flag_certs: String,

    /// Read private key from <key>.  This should be a RSA private key or PKCS8-encoded private key, in PEM format.
    #[structopt(short = "k", long = "key", default_value = "key.pem")]
    flag_key: String,

    /// Read DER-encoded OCSP response from OCSPFILE and staple to certificate.  Optional.
    #[structopt(short = "o", long = "ocsp", default_value = "")]
    flag_ocsp: String,

    /// Negotiate <proto> using ALPN. May be used multiple times.
    #[structopt(short = "p", long = "proto")]
    flag_proto: Vec<String>,

    /// Redis DB connection url.
    #[structopt(short = "d", default_value = "localhost:6379")]
    db: String,
}

/// Simple logger service, it just prints fact of the new connections
// fn logger<T: AsyncRead + AsyncWrite + fmt::Debug>(
//     stream: T,
// ) -> impl Future<Item = T, Error = ()> {
//     info!("New connection: {:?}", stream);
//     ok(stream)
// }

fn setport(s: &str, port: u16) -> String {
    let offset = s.find(':').unwrap_or(s.len());
    (&s[..=offset]).to_owned() + &port.to_string()
}

fn main() -> io::Result<()> {
    let mut lfd = ListenFd::from_env();
    let opt = Opt::from_args();
    if opt.debug {
        env::set_var("RUST_LOG", "actix_net=trace");
    }
    env_logger::init();

    let sys = System::new("VIS");
    let cfg = tls::config(&opt);

    let db = opt.db.clone();

    // let tcpserver = ws::WsServer::default().start();
    // Arbiter::new().exec(move || { session::TcpServer::new(setport(addr, opt.tcpport)); Ok::<_, ()>(())});
    // tcpserver.
    let wsserver = WsServer::default().start();

    let server = HttpServer::new(move || {
        App::new().data(RedisActor::start(db.clone()))
          .wrap(RedisSession::new(db.clone(), &[0; 32]))
          .service(web::resource("/db").route(web::get().to_async(db_route)))
          .data(wsserver.clone())
          .wrap(middleware::Logger::default())
          .service(web::resource("/ws").to(ws_route))
          .service(web::resource("/ws").to(ws_route))
          .default_service(
              Files::new("/", "static")
                  .index_file("index.html")
                  .default_handler(web::to(|| HttpResponse::NotFound().body("File not found"))),
          )
    });

    // let onserv = move || {
    //   HttpService::new(
    //     App::new()
    //       .data(RedisActor::start(db.clone()))
    //       .wrap(middleware::Logger::default())
    //       .wrap(RedisSession::new(db.clone(), &[0;32]))
    //       .default_service(
    //         web::resource("/")
    //           .route(web::get().to_async(root_handler))
    //       )
    //   )
    //   // HttpService::new(AppFactory)
    //   // HttpService::build()
    //     // .keep_alive(0)
    //     // .h1(AppFactory)
    //   // let num = num.clone();
    //   // let acceptor = acceptor.clone();

    //   // HttpService::build()
    //     // .keep_alive(0)
    //     // .upgrade(WSFactory)
    //     // .finish(|_req:Request|
    //     //   _req
    //     //     .map_err(|e| println!("Openssl error: {}", e))
    //     //     .and_then(move |_| {
    //     //       let num = num.fetch_add(1, Ordering::Relaxed);
    //     //       println!("got ssl connection {:?}", num);
    //     //       future::ok(())
    //     //     })
    //     //     .and_then(logger)
    //     // )
    //     // .h2(AppFactory)
    //   // // service for converting incoming TcpStream to a SslStream<TcpStream>
    //   // service_fn(move |stream: Io<TcpStream>| {

    //   //   // SslAcceptorExt::accept_async(&acceptor, stream.into_parts().0)
    //   //   //   .map_err(|e| println!("Openssl error: {}", e))
    //   // })
    //   // // .and_then() combinator uses other service to convert incoming `Request` to a
    //   // // `Response` and then uses that response as an input for next
    //   // // service. in this case, on success we use `logger` service
    //   // .and_then(logger)
    //   // // Next service counts number of connections
    //   // .and_then(move |_| {
    //   //     let num = num.fetch_add(1, Ordering::Relaxed);
    //   //     println!("got ssl connection {:?}", num);
    //   //     future::ok(())
    //   // })
    // };
    let addr = opt.addr.clone(); //.to_socket_addrs().expect("address is not in right format nor resolvable").next().unwrap();
    let raddr: String = setport(&addr, opt.rport);
    {
        let s: &'static str = Box::leak(addr.into_boxed_str());
        let redirector = HttpServer::new(move || {
            App::new().default_service(web::to(move || {
                let loc: String = format!("https://{}", s);
                HttpResponse::MovedPermanently()
                    .set_header(header::LOCATION, loc)
                    .finish()
            }))
        });

        if let Some(l) = lfd.take_tcp_listener(0).unwrap() {
            if let Some(l) = lfd.take_tcp_listener(1).unwrap() {
                redirector.listen(l)?.start();
            } else {
                redirector.bind(raddr.as_str())?.start();
            }
            server.listen_rustls(l, cfg)?
        } else {
            redirector.bind(raddr.as_str())?.start();
            server.bind_rustls(opt.addr, cfg)?
        }
        .start();
    }

    sys.run()
}
