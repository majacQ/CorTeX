// Copyright 2015 Deyan Ginev. See the LICENSE
// file at the top-level directory of this distribution.
//
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed
// except according to those terms.
extern crate zmq;
use zmq::{Error, Message, Context};

pub trait Worker {
  fn work(&self, &Message) -> Message;
  fn name(&self) -> String;
  fn source(&self) -> String;
  fn sink(&self) -> String;

  fn start(&self, limit : Option<i32>) -> Result<(), Error> {
    let mut work_counter = 0;
    // Connect to a task ventilator
    let mut context_source = Context::new();
    let mut source = context_source.socket(zmq::REQ).unwrap();
    assert!(source.connect(&self.source()).is_ok());
    // Connect to a task sink
    let mut context_sink = Context::new();
    let mut sink = context_sink.socket(zmq::PUSH).unwrap();
    assert!(sink.connect(&self.sink()).is_ok());
    // Work in perpetuity
    let mut recv_msg = Message::new().unwrap();
    loop {
      source.send_str(&self.name(), 0).unwrap();
      source.recv(&mut recv_msg, 0).unwrap();
      let payload = self.work(&recv_msg);
      sink.send_msg(payload, 0).unwrap();

      work_counter += 1;
      match limit {
        Some(upper_bound) => {
          if work_counter >= upper_bound {
            break;
          }
        },
        None => {}
      };
    }
    Ok(())
  }
}
pub struct EchoWorker {
  pub name : String,
  pub version : f32,
  pub source : String,
  pub sink : String
}
impl Default for EchoWorker {
  fn default() -> EchoWorker {
    EchoWorker {
      name: "echo_worker".to_string(),
      version: 0.1,
      source: "tcp://localhost:5555".to_string(),
      sink: "tcp://localhost:5556".to_string()      
    }
  }
}
impl Worker for EchoWorker {
  fn name(&self) -> String {self.name.clone()}
  fn source(&self) -> String {self.source.clone()}
  fn sink(&self) -> String {self.sink.clone()}

  fn work(&self, message : &Message) -> Message {
    let payload = message.as_str().unwrap();
    println!("Worker Received: {}", payload);
    Message::from_slice(payload.as_bytes()).unwrap()
  }
}