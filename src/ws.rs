use actix::prelude::*;

// use actix_files as fs;
use actix_web::{web::{Data, Payload}, /* App, */ Error, HttpRequest, HttpResponse /* HttpServer */};
use actix_web_actors::ws::{self, start, Message, ProtocolError};
use std::time::{Duration, Instant};
// use std::sync::{Mutex, Arc};

use crate::{
  server::{WsServer, Connect, Disconnect, CloseAll},
  ws_var::{HEARTBEAT_INTERVAL, CLIENT_TIMEOUT}
};

/// Entry point for our route
pub fn ws_route(
    req: HttpRequest,
    stream: Payload,
    srv: Data<Addr<WsServer>>,
) -> Result<HttpResponse, Error> {
    println!("new websocket requested from {}", req.peer_addr().unwrap());
    start(
        WsSession {
            // id: 0,
            hb: Instant::now(),
            // room: "Main".to_owned(),
            // name: None,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

pub struct WsSession {
    /// unique session id
    // id: usize,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    // /// joined room
    // room: String,
    // /// peer name
    // name: Option<String>,
    // /// Chat server
    addr: Addr<WsServer>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);
        let addr = ctx.address();
        // println!("new websocket connected");
        self.addr.do_send(Connect { addr }); 
        
        // // synchronous style
        // // register self in chat server. `AsyncContext::wait` register
        // // future within context, but context waits until this future resolves
        // // before processing any other events.
        // // HttpContext::state() is instance of WsChatSessionState, state is shared
        // // across all routes within application
        //self.addr.send(Connect {addr}).into_actor(self).then(|_r, _a, _c|  ok(()) ).wait(ctx);
        //   .into_actor(self)
        //   .then(|res, act, ctx| {
        //     match res {
        //       Ok(res) => act.id = res,
        //       // something is wrong with chat server
        //       _ => ctx.stop(),
        //     }
        //     ok(())
        //   })
        //   .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        // notify chat server
        self.addr.do_send(Disconnect {
            addr: ctx.address(),
        });
        Running::Stop
    }
}

#[derive(Message)]
pub struct Close;
impl Handler<Close> for WsSession {
    type Result = ();
    fn handle(&mut self, _: Close, ctx: &mut Self::Context) -> Self::Result {
        println!("WsSession received close message");
        ctx.stop();
    }
}

/// WebSocket message handler
impl StreamHandler<Message, ProtocolError> for WsSession {
    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        // println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Message::Pong(_) => {
                self.hb = Instant::now();
            }
            Message::Text(text) => {
                // println!("message received from {} : {}", self.id, text);
                println!("text received : {}", text);
                if text == "restart_all" {
                    println!("restart all websockets!");
                    self.addr.do_send(CloseAll);
                    ctx.stop();
                }
            }
            /*{
              let m = text.trim();
              // we check for /sss type of messages
              if m.starts_with('/') {
                let v: Vec<&str> = m.splitn(2, ' ').collect();
                match v[0] {
                  "/list" => {
                    // Send ListRooms message to chat server and wait for
                    // response
                    println!("List rooms");
                    self.addr
                      .send(server::ListRooms)
                      .into_actor(self)
                      .then(|res, _, ctx| {
                        match res {
                          Ok(rooms) => {
                            for room in rooms {
                              ctx.text(room);
                            }
                          }
                          _ => println!("Something is wrong"),
                        }
                        fut::ok(())
                      })
                      .wait(ctx)
                    // .wait(ctx) pauses all events in context,
                    // so actor wont receive any new messages until it get list
                    // of rooms back
                  }
                  "/join" => {
                    if v.len() == 2 {
                      self.room = v[1].to_owned();
                      self.addr.do_send(server::Join {
                        id: self.id,
                        name: self.room.clone(),
                      });

                      ctx.text("joined");
                    } else {
                      ctx.text("!!! room name is required");
                    }
                  }
                  "/name" => {
                    if v.len() == 2 {
                      self.name = Some(v[1].to_owned());
                    } else {
                      ctx.text("!!! name is required");
                    }
                  }
                  _ => ctx.text(format!("!!! unknown command: {:?}", m)),
                }
              } else {
                let msg = if let Some(ref name) = self.name {
                  format!("{}: {}", name, m)
                } else {
                  m.to_owned()
                };
                // send message to chat server
                self.addr.do_send(server::Message {
                  id: self.id,
                  msg: msg,
                  room: self.room.clone(),
                })
              }
            } */
            Message::Binary(_) => println!("Unexpected binary"),
            Message::Close(_) => {
                ctx.stop();
            }
            Message::Nop => (),
        }
    }
}

impl WsSession {
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                act.addr.do_send(Disconnect {
                    addr: ctx.address(),
                });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping("");
        });
    }
}
