// Copyright 2015-2018 Deyan Ginev. See the LICENSE
// file at the top-level directory of this distribution.
//
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed
// except according to those terms.

extern crate tempfile;
extern crate zmq;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use backend::DEFAULT_DB_ADDRESS;
use dispatcher::finalize::Finalize;
use dispatcher::sink::Sink;
use dispatcher::ventilator::Ventilator;
use helpers::{TaskProgress, TaskReport};
use models::Service;
use zmq::Error;

/// Manager struct responsible for dispatching and receiving tasks
pub struct TaskManager {
  /// port for requesting/dispatching jobs
  pub source_port: usize,
  /// port for responding/receiving results
  pub result_port: usize,
  /// the size of the dispatch queue
  /// (also the batch size for Task store queue requests)
  pub queue_size: usize,
  /// size of an individual message chunk sent via zeromq
  /// (keep this small to avoid large RAM use, increase to reduce network bandwidth)
  pub message_size: usize,
  /// address for the Task store postgres endpoint
  pub backend_address: String,
}

impl Default for TaskManager {
  fn default() -> TaskManager {
    TaskManager {
      source_port: 5555,
      result_port: 5556,
      queue_size: 100,
      message_size: 100_000,
      backend_address: DEFAULT_DB_ADDRESS.to_string(),
    }
  }
}

impl TaskManager {
  /// Starts a new manager, spinning of dispatch/sink servers, listening on the specified ports
  pub fn start(&self, job_limit: Option<usize>) -> Result<(), Error> {
    // We'll use some local memoization shared between source and sink:
    let services: HashMap<String, Option<Service>> = HashMap::new();
    let progress_queue: HashMap<i64, TaskProgress> = HashMap::new();
    let done_queue: Vec<TaskReport> = Vec::new();

    let services_arc = Arc::new(Mutex::new(services));
    let progress_queue_arc = Arc::new(Mutex::new(progress_queue));
    let done_queue_arc = Arc::new(Mutex::new(done_queue));

    // First prepare the source ventilator
    let source_port = self.source_port;
    let source_queue_size = self.queue_size;
    let source_message_size = self.message_size;
    let source_backend_address = self.backend_address.clone();

    let vent_services_arc = services_arc.clone();
    let vent_progress_queue_arc = progress_queue_arc.clone();
    let vent_done_queue_arc = done_queue_arc.clone();
    let vent_thread = thread::spawn(move || {
      Ventilator {
        port: source_port,
        queue_size: source_queue_size,
        message_size: source_message_size,
        backend_address: source_backend_address.clone(),
      }.start(
        vent_services_arc,
        vent_progress_queue_arc,
        vent_done_queue_arc,
        job_limit,
      )
        .unwrap();
    });

    // Next prepare the finalize thread which will persist finished jobs to the DB
    let finalize_backend_address = self.backend_address.clone();
    let finalize_done_queue_arc = done_queue_arc.clone();
    let finalize_thread = thread::spawn(move || {
      Finalize {
        backend_address: finalize_backend_address.clone(),
        job_limit,
      }.start(finalize_done_queue_arc);
    });

    // Now prepare the results sink
    let result_port = self.result_port.clone();
    let result_queue_size = self.queue_size;
    let result_message_size = self.message_size;
    let result_backend_address = self.backend_address.clone();

    let sink_services_arc = services_arc.clone();
    let sink_progress_queue_arc = progress_queue_arc.clone();

    let sink_done_queue_arc = done_queue_arc.clone();
    let sink_thread = thread::spawn(move || {
      Sink {
        port: result_port,
        queue_size: result_queue_size,
        message_size: result_message_size,
        backend_address: result_backend_address.clone(),
      }.start(
        sink_services_arc,
        sink_progress_queue_arc,
        sink_done_queue_arc,
        job_limit,
      )
        .unwrap();
    });

    if vent_thread.join().is_err() {
      println!("Ventilator thread died unexpectedly!");
      Err(zmq::Error::ETERM)
    } else if sink_thread.join().is_err() {
      println!("Sink thread died unexpectedly!");
      Err(zmq::Error::ETERM)
    } else if finalize_thread.join().is_err() {
      println!("DB thread died unexpectedly!");
      Err(zmq::Error::ETERM)
    } else {
      println!("Manager successfully terminated!");
      Ok(())
    }
  }
}
