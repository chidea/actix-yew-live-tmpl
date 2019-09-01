// use std::{
// hash::Hash,
// str,
// io::Write,
// net::{SocketAddr, IpAddr, Ipv4Addr},
// sync::Mutex,
// time::{Instant}
// };
// use actix_http::{
// body::Body,
// http::{
// header::{CONTENT_TYPE, SERVER},
// HeaderValue,
// StatusCode,
// },
// Error, Request, Response,
// };
// use actix_service::{
// NewService,
// Service,
// };
// use actix_server::{ServerConfig};
// use actix_web::dev::Server
use actix::prelude::*;
// use bytes::{BytesMut, Bytes};
// use futures::{
// future::{
// ok,
// join_all,
// Future,
// },
// Async, Poll,
// };
// use serde_json::to_writer;
// use actix_web::{
// App,
// web,
// middleware,
// Error as AWError,
// HttpResponse,
// HttpRequest,
// HttpServer,
// };
// use actix_web_actors::ws::{Message as WsMessage, CloseCode, CloseReason };
// use askama::Template;
//use actix_redis::{Command, RedisActor, Error as ARError};
use actix_redis::{RedisActor};
// use redis_async::{
// client::{PairedConnection, paired_connect, PubsubConnection, pubsub_connect},
// resp::{RespValue},
// };

use crate::ws::{Close as WsClose, WsSession};
// use super::db::{RedisConnection};

// pub struct App {
//     // db: PgConnection,
//     db: RedisConnection,
//     // db: Arc<PairedConnection>,
//     hdr_srv: HeaderValue,
//     hdr_ctjson: HeaderValue,
//     hdr_cthtml: HeaderValue,
// }

// impl Service for App {
//     type Request = Request;
//     type Response = Response;
//     type Error = Error;
//     type Future = Box<dyn Future<Item = Response, Error = Error>>;

//     #[inline]
//     fn poll_ready(&mut self) -> Poll<(), Self::Error> {
//         Ok(Async::Ready(()))
//     }

//     fn call(&mut self, req: Request) -> Self::Future {
//         let path = req.path();
//         match path {
//             "/db" => {
//               let h_srv = self.hdr_srv.clone();
//               let h_ct = self.hdr_ctjson.clone();

//               Box::new(self.db.get("mydomain:one")
//                 .map(|v:String| {
//                   let mut body = BytesMut::new();
//                   serde_json::to_writer(Writer(&mut body), &Message{
//                     message: &*v
//                   }).unwrap();

//                   let mut res = Response::with_body(StatusCode::OK, Body::Bytes(body.freeze()));
//                   let hdrs = res.headers_mut();
//                   hdrs.insert(SERVER, h_srv);
//                   hdrs.insert(CONTENT_TYPE, h_ct);
//                   res
//                 })
//               )
//             }
//             "/fortune" => {
//               let h_srv = self.hdr_srv.clone();
//               let h_ct = self.hdr_cthtml.clone();

//               // Box::new(self.db.tell_fortune().from_err().map(move |fortunes| {
//               Box::new(ok({
//                 let mut body = BytesMut::with_capacity(2048);
//                 let mut writer = Writer(&mut body);
//                 let _ = write!(writer, "{}", HelloTemplate { name : "tester" });//FortunesTemplate { fortunes });
//                 let mut res = Response::with_body(StatusCode::OK, Body::Bytes(body.freeze()));
//                 let hdrs = res.headers_mut();
//                 hdrs.insert(SERVER, h_srv);
//                 hdrs.insert(CONTENT_TYPE, h_ct);
//                 res
//               }))
//             }
//             "/json" => {
//               Box::new(ok(json()))
//             }
//             "/plaintext" => {
//               Box::new(ok(plaintext()))
//             }
//             // "/queries" => {
//             //     let q = utils::get_query_param(req.uri().query().unwrap_or("")) as usize;
//             //     let h_srv = self.hdr_srv.clone();
//             //     let h_ct = self.hdr_ctjson.clone();

//             //     Box::new(self.db.get_worlds(q).from_err().map(move |worlds| {
//             //         let mut body = BytesMut::with_capacity(35 * worlds.len());
//             //         to_writer(Writer(&mut body), &worlds).unwrap();
//             //         let mut res =
//             //             Response::with_body(StatusCode::OK, Body::Bytes(body.freeze()));
//             //         let hdrs = res.headers_mut();
//             //         hdrs.insert(SERVER, h_srv);
//             //         hdrs.insert(CONTENT_TYPE, h_ct);
//             //         res
//             //     }))
//             // }
//             // "/updates" => {
//             //     let q = utils::get_query_param(req.uri().query().unwrap_or("")) as usize;
//             //     let h_srv = self.hdr_srv.clone();
//             //     let h_ct = self.hdr_ctjson.clone();

//             //     Box::new(self.db.update(q).from_err().map(move |worlds| {
//             //         let mut body = BytesMut::with_capacity(35 * worlds.len());
//             //         to_writer(Writer(&mut body), &worlds).unwrap();
//             //         let mut res =
//             //             Response::with_body(StatusCode::OK, Body::Bytes(body.freeze()));
//             //         let hdrs = res.headers_mut();
//             //         hdrs.insert(SERVER, h_srv);
//             //         hdrs.insert(CONTENT_TYPE, h_ct);
//             //         res
//             //     }))
//             // }
//             _ => Box::new(ok(Response::new(StatusCode::NOT_FOUND))),
//         }
//     }
// }

// #[derive(Clone)]
// pub struct AppFactory;

// impl NewService for AppFactory {
//   type Config = ServerConfig;
//   type Request = Request;
//   type Response = Response;
//   type Error = Error;
//   type Service = App;
//   type InitError = ();
//   type Future = Box<dyn Future<Item = Self::Service, Error = Self::InitError>>;

//   fn new_service(&self, _: &ServerConfig) -> Self::Future {
//     // const DB_URL: &str =
//     //     "postgres://benchmarkdbuser:benchmarkdbpass@tfb-database/hello_world";

//     // Box::new(PgConnection::connect(DB_URL).map(|db| App {
//     //     db,
//     //     hdr_srv: HeaderValue::from_static("Actix"),
//     //     hdr_ctjson: HeaderValue::from_static("application/json"),
//     //     hdr_cthtml: HeaderValue::from_static("text/html; charset=utf-8"),
//     // }));
//     Box::new(
//         // paired_connect(&String::from(DB_URL).parse().unwrap())
//         RedisConnection::connect(DB_URL)
//           .map_err(|_| ())
//           .map(|db|{
//             let app = App {
//               db,
//               hdr_srv: HeaderValue::from_static("Actix"),
//               hdr_ctjson: HeaderValue::from_static("application/json"),
//               hdr_cthtml: HeaderValue::from_static("text/html; charset=utf-8"),
//             };
//             app
//           })
//       // })
//     )
//   }
// }

// pub fn json() -> HttpResponse {
//   let message = Message {
//       message: "Hello, World!",
//   };
//   let mut body = BytesMut::with_capacity(SIZE);
//   serde_json::to_writer(Writer(&mut body), &message).unwrap();

//   let mut res = HttpResponse::with_body(StatusCode::OK, Body::Bytes(body.freeze()));
//   res.headers_mut()
//       .insert(SERVER, HeaderValue::from_static("Actix"));
//   res.headers_mut()
//       .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
//   res
// }

// fn plaintext() -> HttpResponse {
//   let mut res = HttpResponse::with_body(
//     StatusCode::OK,
//     Body::Bytes(Bytes::from_static(b"Hello, World!")),
//   );
//   res.headers_mut()
//     .insert(SERVER, HeaderValue::from_static("Actix"));
//   res.headers_mut()
//     .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
//   res
// }

// #[derive(Template)]
// #[template(path = "test.html")]
// struct HelloTemplate<'a> {
//   name: &'a str,
// }

// pub fn root_handler(
//   req: web::HttpRequest
// ) -> impl Future<Item = HttpResponse, Error = ()> {
//   let path = req.match_info().query("filename").parse().unwrap();
//   HttpResponse::from(
//     Ok( NamedFile::open(path).unwrap() )
//   )
//   // ok( HttpResponse::Ok().body("hi"))
//   //       Ok(HttpResponse::InternalServerError().finish())
// }

pub struct WsServer {
    sessions: Vec<Addr<WsSession>>,
    db: Addr<RedisActor>,
}
impl Actor for WsServer {
    type Context = Context<Self>;
}
impl WsServer {
    pub fn new(db : Addr<RedisActor>) -> WsServer {
        let sessions = vec![];
        WsServer { sessions, db }
    }
}

impl WsServer {
    fn close_all(&self) {
        // for s in &*self.sessions.lock().unwrap() {
        for s in &self.sessions {
            // if let Some(v) = s.upgrade(){
            if s.connected() {
                // println!("sending WsClose");
                // v.do_send(WsClose);
                s.do_send(WsClose);
                //WsMessage::Close(Some(CloseReason { code: CloseCode::Restart, description: None })));
            }
        }
    }
}

/// new websocket connection
#[derive(Message)]
pub struct Connect {
    pub addr: Addr<WsSession>,
}
// impl Message for Connect {
//   type Result = usize;
// }

impl Handler<Connect> for WsServer {
    type Result = ();
    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        // println!("{:?} joined wsserver", msg.addr);
        // let mut s = &mut *self.sessions.get_mut().unwrap();
        let s = &mut self.sessions;
        s.push(msg.addr); //.downgrade());
        println!(
            "new web socket added to server : {} sockets opened",
            s.len()
        );
    }
}

/// websocket session disconnected
#[derive(Message)]
pub struct Disconnect {
    pub addr: Addr<WsSession>,
    // pub id : usize,
}
impl Handler<Disconnect> for WsServer {
    type Result = ();
    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        println!("a websocket session requested disconnect");
        let mut s = 0;
        let mut f = false;
        // let mut ss = &mut *self.sessions.get_mut().unwrap();
        let ss = &mut self.sessions;
        for i in 0..ss.len() {
            // if let Some(v) = self.sessions[i].upgrade(){
            if ss[i] == msg.addr {
                // if ss[i] == msg.addr {
                // if v == msg.addr {
                s = i;
                f = true;
                break;
                // }
            }
        }
        if f {
            ss.remove(s);
            println!(
                "a websocket session removed from server : {} sockets opened",
                ss.len()
            );
        }
    }
}
/// request to close all other connections
#[derive(Message)]
pub struct CloseAll;
impl Handler<CloseAll> for WsServer {
    type Result = ();
    fn handle(&mut self, _msg: CloseAll, _ctx: &mut Self::Context) -> Self::Result {
        println!("received CloseAll");
        self.close_all();
    }
}
