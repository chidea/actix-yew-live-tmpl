use futures::{Future, future::AndThen};
use redis_async::resp::RespValue;
// use std::{
// io, sync::Arc,
// net::{SocketAddr}
// };
use actix::prelude::*;
// use actix_http::Error;
use actix_redis::{Command, Error as ARError, RedisActor};
use actix_web::{
    // App,
    web::Data,
    // middleware,
    Error as AWError,
    HttpResponse,
    // HttpRequest,
    // HttpServer,
};
// use crate::util::Message as DBMessage;
// use super::util::{Writer, Message, SIZE};
// use bytes::{BytesMut, Bytes};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct DBMessage<'a> {
    message: &'a str,
}

pub fn db_route(
    redis: Data<Addr<RedisActor>>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
  let set = redis.send(Command(resp_array!["SET", "mydomain:one", "dbtest.1"]));
  let get = redis.send(Command(resp_array!["GET", "mydomain:one"]));
  
  set.join(get).map_err(AWError::from).and_then(|r| match &r {
    (_,g) => match g {
        Ok(RespValue::BulkString(s)) => {
          Ok(HttpResponse::Ok().json(DBMessage {
              message: std::str::from_utf8(s).unwrap(),
          }))
        }
        _ => {
          // println!("---->{:?}", e);
          Ok(HttpResponse::InternalServerError().finish())
        }
    }
    // _ => Ok(HttpResponse::InternalServerError().finish())
  })
  // match set.map_err(AWError::from).wait().unwrap() {
  //   Ok(_) => {
  //     get.map_err(AWError::from)
  //       .and_then(|res: Result<RespValue, ARError>| match &res {
  //           Ok(RespValue::BulkString(s)) => {
  //               Ok(HttpResponse::Ok().json(DBMessage {
  //                   message: std::str::from_utf8(s).unwrap(),
  //               }))
  //               // .body(String::from_utf8(s.to_vec()).unwrap()))
  //           }
  //           _ => {
                
  //           }
  //       })
  //   }
  //   _ => {
  //   }
  // }
  
  
    // redis
    //   .send(Command(resp_array![
    //     "SET",
    //     "mydomain:one",
    //     "chaos.1"]))
    //   // .and_then
    //   //   "mydomain:two",
    //   //   "chaos.2",
    //   //   "mydomain:three",
    //   //   "chaos.3"
    //   // ]))
    //   // .send(Command(resp_array![
    //   //     "DEL",
    //   //     "mydomain:one",
    //   //     "mydomain:two",
    //   //     "mydomain:three"
    //   // ]))
    //   .map_err(AWError::from)
    //   .and_then(|res: Result<RespValue, ARError>| match &res {
    //     Ok(RespValue::Integer(x)) if x == &1 => {
    //       Ok(HttpResponse::Ok().body("successfully set values"))
    //     }
    //     _ => {
    //       println!("---->{:?}", res);
    //       Ok(HttpResponse::InternalServerError().finish())
    //     }
    //   })
}
/*
// #[derive(Copy, Clone)]
pub struct RedisConnection {
  pub paired : Arc<PairedConnection>,
  pubsub : Arc<PubsubConnection>,
}

impl Actor for RedisConnection {
  type Context = Context<Self>;
}

impl Future for RedisConnection {
  type Item = RedisConnection;
  type Error = ();
  fn poll (self:&mut Self) -> Result<Async<Self::Item>, Self::Error> {
    Ok(Async::Ready(RedisConnection{
      paired: self.paired.clone(),
      pubsub: self.pubsub.clone(),
    }))
  }
}
impl RedisConnection {
  pub fn connect(db_url: &str) -> impl Future<Item = RedisConnection, Error= ()> {
    // let addr : SocketAddr = String::from(db_url).parse().unwrap();
    // paired_connect(&addr)
    //   .join(pubsub_connect(&addr))
    //   .and_then(||{

    //   })
    // join_all(vec![addr.into_future(), addr.clone().into_future()])
      // .map(move |mut addrs|{
        // let (a, b) = (addrs.pop().unwrap(), addrs.pop().unwrap());
        let paired = paired_connect(&String::from(db_url).parse().unwrap())
          .map_err(|e| panic!("cannot connect to redis : {}", e))
          .map(|p|Arc::new(p));
        let pubsub = pubsub_connect(&String::from(db_url).parse().unwrap())
          .map_err(|e| panic!("cannot connect to redis pubsub : {}", e))
          .map(|p|Arc::new(p));
        paired.join(pubsub).and_then(|(paired, pubsub)|{
          RedisConnection{
            paired,
            pubsub,
          }
        })
          // .and_then(move |paired| {
            // pubsub_connect(&String::from(db_url).parse().unwrap())
              // .map_err(|e| panic!("cannot connect to redis pubsub : {}", e))
              // .and_then(move |pubsub|{

              // })
          // })
      // })
  }
}
impl RedisConnection {
  pub fn get(&mut self, key:&str) -> impl Future<Item = String, Error = Error> {
    let db = self.paired.clone();
    db.send(resp_array!["GET", key])
      .map_err(|e| Error::from(io::Error::new(io::ErrorKind::Other, format!("{:?}", e))))
      .map(move |v:String| {
        v
        // let mut body = BytesMut::new();
        // serde_json::to_writer(Writer(&mut body), &Message{
        //   message: v.as_str()
        // }).unwrap();
        // body.freeze()
      })
  }
} */
