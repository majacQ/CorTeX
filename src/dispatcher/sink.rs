extern crate tempfile;
extern crate zmq;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use time;

use dispatcher::server;
use helpers;
use helpers::{TaskProgress, TaskReport, TaskStatus};
use models::Service;
use zmq::Error;

/// Specifies the binding and operation parameters for a ZMQ sink component
pub struct Sink {
  /// port to listen on
  pub port: usize,
  /// the size of the dispatch queue
  /// (also the batch size for Task store queue requests)
  pub queue_size: usize,
  /// size of an individual message chunk sent via zeromq
  /// (keep this small to avoid large RAM use, increase to reduce network bandwidth)
  pub message_size: usize,
  /// address for the Task store postgres endpoint
  pub backend_address: String,
}

impl Sink {
  /// Starts a receiver/sink `Server` (ZMQ Pull), to accept processing responses.
  /// The sink shares state with other manager threads via queues for tasks in progress,
  /// as well as a queue for completed tasks pending persisting to disk.
  /// A job limit can be provided as a termination condition for the sink server.
  pub fn start(
    &self,
    services_arc: Arc<Mutex<HashMap<String, Option<Service>>>>,
    progress_queue_arc: Arc<Mutex<HashMap<i64, TaskProgress>>>,
    done_queue_arc: Arc<Mutex<Vec<TaskReport>>>,
    job_limit: Option<usize>,
  ) -> Result<(), Error>
  {
    // Ok, let's bind to a port and start broadcasting
    let context = zmq::Context::new();
    let sink = context.socket(zmq::PULL).unwrap();
    let port_str = self.port.to_string();
    let address = "tcp://*:".to_string() + &port_str;
    assert!(sink.bind(&address).is_ok());

    let mut sink_job_count: usize = 0;

    loop {
      let mut recv_msg = zmq::Message::new().unwrap();
      let mut taskid_msg = zmq::Message::new().unwrap();
      let mut service_msg = zmq::Message::new().unwrap();

      sink.recv(&mut service_msg, 0).unwrap();
      let service_name = match service_msg.as_str() {
        Some(some_name) => some_name,
        None => "_unknown_",
      };

      sink.recv(&mut taskid_msg, 0).unwrap();
      let taskid_str = match taskid_msg.as_str() {
        Some(some_id) => some_id,
        None => "-1",
      };
      let taskid = match taskid_str.parse::<i64>() {
        Ok(some_id) => some_id,
        Err(_) => -1,
      };
      // We have a job, count it
      sink_job_count += 1;
      let mut total_incoming = 0;
      let request_time = time::get_time();
      println!(
        "Incoming sink job {:?} for Service: {:?}, taskid: {:?}",
        sink_job_count, service_name, taskid_str
      );

      if let Some(task_progress) = server::pop_progress_task(&progress_queue_arc, taskid) {
        let task = task_progress.task;
        let service_option = server::get_service(service_name, &services_arc);
        match service_option.clone() {
          None => {
            println!("Error TODO: Server::get_service found nothing.");
          }, // TODO: Handle errors
          Some(service) => {
            if service.id == task.service_id {
              // println!("Task and Service match up.");
              if service.id == 1 {
                // No payload needed for init
                match sink.recv(&mut recv_msg, 0) {
                  Ok(_) => {},
                  Err(e) => {
                    println!("Error TODO: sink.recv failed: {:?}", e);
                  },
                };
                let done_report = TaskReport {
                  task: task.clone(),
                  status: TaskStatus::NoProblem,
                  messages: Vec::new(),
                };
                server::push_done_queue(&done_queue_arc, done_report);
              } else {
                // Receive the rest of the input in the correct file
                match Path::new(&task.entry.clone()).parent() {
                  None => {
                    println!("Error TODO: Path::new(&task.entry).parent() failed.");
                  },
                  Some(recv_dir) => {
                    match recv_dir.to_str() {
                      None => {
                        println!("Error TODO: recv_dir.to_str() failed");
                      },
                      Some(recv_dir_str) => {
                        let recv_dir_string = recv_dir_str.to_string();
                        let recv_pathname = recv_dir_string + "/" + &service.name + ".zip";
                        let recv_path = Path::new(&recv_pathname);
                        // println!("Will write to {:?}", recv_path);
                        {
                          // Explicitly scope file, so that we drop it the moment we are done
                          // writing.
                          let mut file = match File::create(recv_path) {
                            Ok(f) => f,
                            Err(e) => {
                              println!("Error TODO: File::create(recv_path): {:?}", e);
                              continue;
                            },
                          };
                          while let Ok(_) = sink.recv(&mut recv_msg, 0) {
                            // Err(e) => {
                            //   println!("Error TODO: sink.recv (line 309) failed: {:?}", e);
                            // }
                            match file.write(recv_msg.deref()) {
                              Ok(written_bytes) => total_incoming += written_bytes,
                              Err(e) => {
                                println!(
                                  "Error TODO: file.write(recv_msg.deref()) failed: {:?}",
                                  e
                                );
                                break;
                              },
                            };
                            match sink.get_rcvmore() {
                              Ok(true) => {}, // keep receiving
                              _ => break,     /* println!("Error TODO: sink.get_rcvmore failed:
                                                * {:?}", e); */
                            };
                          }
                          drop(file);
                        }
                        // Then mark the task done. This can be in a new thread later on
                        let done_report = helpers::generate_report(task, recv_path);
                        server::push_done_queue(&done_queue_arc, done_report);
                      },
                    }
                  },
                }
              }
            } else {
              // Otherwise just discard the rest of the message
              loop {
                sink.recv(&mut recv_msg, 0).unwrap();
                if !sink.get_rcvmore().unwrap() {
                  break;
                }
              }
            }
          },
        };
      }
      let responded_time = time::get_time();
      let request_duration = (responded_time - request_time).num_milliseconds();
      println!(
        "Sink job {}, message size: {}, took {}ms.",
        sink_job_count, total_incoming, request_duration
      );
      if job_limit.is_some() && (sink_job_count >= job_limit.unwrap()) {
        println!(
          "Manager job limit of {:?} reached, terminating Sink thread...",
          job_limit.unwrap()
        );
        break;
      }
    }
    Ok(())
  }
}