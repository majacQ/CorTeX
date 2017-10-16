// Copyright 2015-2016 Deyan Ginev. See the LICENSE
// file at the top-level directory of this distribution.
//
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed
// except according to those terms.
extern crate cortex;
extern crate postgres;

use cortex::backend;
use cortex::models::NewTask;

#[test]
fn table_ops() {
  let backend = backend::testdb();
  // Delete any mock tasks

  // Add a mock task
  let count_added = backend.add(NewTask{entry: "table_ops_mock_task", serviceid: 1, corpusid:1, status: -5});
  assert_eq!(count_added, Ok(1)); // new task!
  // Get the fields from the DB

  // Delete again and verify deletion works

}

#[test]
fn connection_pool_availability() {
  // Test connections are available
  // Test going over the MAX pool limit of connections is sane
}
