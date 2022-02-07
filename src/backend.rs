// Copyright 2015-2018 Deyan Ginev. See the LICENSE
// file at the top-level directory of this distribution.
//
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed
// except according to those terms.

//! All aggregate operations over the CorTeX PostgresQL store are accessed through the connection of
//! a `Backend` object.

mod corpora_aggregate;
mod mark;
mod reports;
mod services_aggregate;
mod tasks_aggregate;
pub(crate) use reports::progress_report;
pub use reports::TaskReportOptions;

use diesel::pg::PgConnection;
use diesel::result::Error;
use diesel::*;
use dotenv::dotenv;
use std::collections::HashMap;

use crate::concerns::{CortexDeletable, CortexInsertable};
use crate::helpers::{TaskReport, TaskStatus};
use crate::models::{Corpus, NewTask, Service, Task};

lazy_static! {
  static ref ENTRY_NAME_REGEX: Regex = Regex::new(r"^(.+)/[^/]+$").unwrap();
  static ref TASK_REPORT_NAME_REGEX: Regex = Regex::new(r"^.+/(.+)\..+$").unwrap();
}

/// The production database postgresql address, set from the .env configuration file
pub const DEFAULT_DB_ADDRESS: &str = dotenv!("DATABASE_URL");
/// The test database postgresql address, set from the .env configuration file
pub const TEST_DB_ADDRESS: &str = dotenv!("TEST_DATABASE_URL");

/// Provides an interface to the Postgres task store
pub struct Backend {
  /// The Diesel PgConnection object
  pub connection: PgConnection,
}
impl Default for Backend {
  fn default() -> Self {
    dotenv().ok();
    let connection = connection_at(DEFAULT_DB_ADDRESS);

    Backend { connection }
  }
}

/// Constructs a new Task store representation from a Postgres DB address
pub fn connection_at(address: &str) -> PgConnection {
  PgConnection::establish(address).unwrap_or_else(|_| panic!("Error connecting to {}", address))
}
/// Constructs the default Backend struct for testing
pub fn testdb() -> Backend {
  dotenv().ok();
  Backend {
    connection: connection_at(TEST_DB_ADDRESS),
  }
}
/// Constructs a Backend at a given address
pub fn from_address(address: &str) -> Backend {
  Backend {
    connection: connection_at(address),
  }
}

/// Options container for relevant fields in requesting a `(corpus, service)` rerun
pub struct RerunOptions<'a> {
  /// corpus to rerun
  pub corpus: &'a Corpus,
  /// service to rerun
  pub service: &'a Service,
  /// optionally, severity level filter
  pub severity_opt: Option<String>,
  /// optionally, category level filter
  pub category_opt: Option<String>,
  /// optionally, what level filter
  pub what_opt: Option<String>,
  /// optionally, owner of the rerun (default is "admin")
  pub owner_opt: Option<String>,
  /// optionally, description of the rerun (default is "rerun")
  pub description_opt: Option<String>,
}

/// Instance methods
impl Backend {
  /// Insert a vector of new `NewTask` tasks into the Task store
  /// For example, on import, or when a new service is activated on a corpus
  pub fn mark_imported(&self, imported_tasks: &[NewTask]) -> Result<usize, Error> {
    mark::mark_imported(&self.connection, imported_tasks)
  }
  /// Insert a vector of `TaskReport` reports into the Task store, also marking their tasks as
  /// completed with the correct status code.
  pub fn mark_done(&self, reports: &[TaskReport]) -> Result<(), Error> {
    mark::mark_done(&self.connection, reports)
  }
  /// Given a complex selector, of a `Corpus`, `Service`, and the optional `severity`, `category`
  /// and `what` mark all matching tasks to be rerun
  pub fn mark_rerun(&self, options: RerunOptions) -> Result<(), Error> {
    mark::mark_rerun(&self.connection, options)
  }

  /// While not changing any status information for Tasks, add a new historical run bookmark
  pub fn mark_new_run(
    &self,
    corpus: &Corpus,
    service: &Service,
    owner: String,
    description: String,
  ) -> Result<(), Error>
  {
  <<<<<<< loading-info-messages
    use schema::tasks::{corpus_id, service_id, status};
    // Rerun = set status to TODO for all tasks, deleting old logs
    let mark: i32 = random_mark();

    // First, mark as blocked all of the tasks in the chosen scope, using a special mark
    match severity_opt {
      Some(severity) => match category_opt {
        Some(category) => match what_opt {
          // All tasks in a "what" class
          Some(what) => try!(match severity.to_lowercase().as_str() {
            "warning" => LogWarning::mark_rerun_by_what(
              mark,
              corpus.id,
              service.id,
              &category,
              &what,
              &self.connection,
            ),
            "error" => LogError::mark_rerun_by_what(
              mark,
              corpus.id,
              service.id,
              &category,
              &what,
              &self.connection,
            ),
            "fatal" => LogFatal::mark_rerun_by_what(
              mark,
              corpus.id,
              service.id,
              &category,
              &what,
              &self.connection,
            ),
            "invalid" => LogInvalid::mark_rerun_by_what(
              mark,
              corpus.id,
              service.id,
              &category,
              &what,
              &self.connection,
            ),
            _ => LogInfo::mark_rerun_by_what(
              mark,
              corpus.id,
              service.id,
              &category,
              &what,
              &self.connection,
            ),
          }),
          // None: All tasks in a category
          None => try!(match severity.to_lowercase().as_str() {
            "warning" => LogWarning::mark_rerun_by_category(
              mark,
              corpus.id,
              service.id,
              &category,
              &self.connection,
            ),
            "error" => LogError::mark_rerun_by_category(
              mark,
              corpus.id,
              service.id,
              &category,
              &self.connection,
            ),
            "fatal" => LogFatal::mark_rerun_by_category(
              mark,
              corpus.id,
              service.id,
              &category,
              &self.connection,
            ),
            "invalid" => LogInvalid::mark_rerun_by_category(
              mark,
              corpus.id,
              service.id,
              &category,
              &self.connection,
            ),
            _ => LogInfo::mark_rerun_by_category(
              mark,
              corpus.id,
              service.id,
              &category,
              &self.connection,
            ),
          }),
        },
        None => {
          // All tasks in a certain status/severity
          let status_to_rerun: i32 = TaskStatus::from_key(&severity)
            .unwrap_or(TaskStatus::Fatal)
            .raw();
          try!(
            update(tasks::table)
              .filter(corpus_id.eq(corpus.id))
              .filter(service_id.eq(service.id))
              .filter(status.eq(status_to_rerun))
              .set(status.eq(mark))
              .execute(&self.connection)
          )
        },
      },
      None => {
        // Entire corpus
        try!(
          update(tasks::table)
            .filter(corpus_id.eq(corpus.id))
            .filter(service_id.eq(service.id))
            .filter(status.lt(0))
            .set(status.eq(mark))
            .execute(&self.connection)
        )
      },
    };

    // Next, delete all logs for the blocked tasks.
    // Note that if we are using a negative blocking status, this query should get sped up via an
    // "Index Scan using log_taskid on logs"
    let affected_tasks = tasks::table
      .filter(corpus_id.eq(corpus.id))
      .filter(service_id.eq(service.id))
      .filter(status.eq(mark));
    let affected_tasks_ids = affected_tasks.select(tasks::id);

    let affected_log_infos = log_infos::table.filter(log_infos::task_id.eq_any(affected_tasks_ids));
    try!(delete(affected_log_infos).execute(&self.connection));
    let affected_log_warnings =
      log_warnings::table.filter(log_warnings::task_id.eq_any(affected_tasks_ids));
    try!(delete(affected_log_warnings).execute(&self.connection));
    let affected_log_errors =
      log_errors::table.filter(log_errors::task_id.eq_any(affected_tasks_ids));
    try!(delete(affected_log_errors).execute(&self.connection));
    let affected_log_fatals =
      log_fatals::table.filter(log_fatals::task_id.eq_any(affected_tasks_ids));
    try!(delete(affected_log_fatals).execute(&self.connection));
    let affected_log_invalids =
      log_invalids::table.filter(log_invalids::task_id.eq_any(affected_tasks_ids));
    try!(delete(affected_log_invalids).execute(&self.connection));

    // Lastly, switch all blocked tasks to TODO, and complete the rerun mark pass.
    try!(
      update(affected_tasks)
        .set(status.eq(TaskStatus::TODO.raw()))
        .execute(&self.connection)
    );

    Ok(())
  =======
    mark::mark_new_run(
      &self.connection,
      corpus,
      service,
      owner,
      description,
    )
  >>>>>>> dependabot/cargo/sys-info-0.9.0
  }

  /// Generic delete method, uses primary "id" field
  pub fn delete<Model: CortexDeletable>(&self, object: &Model) -> Result<usize, Error> {
    object.delete_by(&self.connection, "id")
  }
  /// Delete all entries matching the "field" value of a given object
  pub fn delete_by<Model: CortexDeletable>(
    &self,
    object: &Model,
    field: &str,
  ) -> Result<usize, Error>
  {
    object.delete_by(&self.connection, field)
  }
  /// Generic addition method, attempting to insert in the DB a Task store datum
  /// applicable for any struct implementing the `CortexORM` trait
  /// (for example `Corpus`, `Service`, `Task`)
  pub fn add<Model: CortexInsertable>(&self, object: &Model) -> Result<usize, Error> {
    object.create(&self.connection)
  }

  /// Fetches no more than `limit` queued tasks for a given `Service`
  pub fn fetch_tasks(&self, service: &Service, limit: usize) -> Result<Vec<Task>, Error> {
    tasks_aggregate::fetch_tasks(&self.connection, service, limit)
  }
  /// Globally resets any "in progress" tasks back to "queued".
  /// Particularly useful for dispatcher restarts, when all "in progress" tasks need to be
  /// invalidated
  pub fn clear_limbo_tasks(&self) -> Result<usize, Error> {
    tasks_aggregate::clear_limbo_tasks(&self.connection)
  }

  /// Activates an existing service on a given corpus (via PATH)
  /// if the service has previously been registered, this call will `RESET` the service into a mint
  /// state also removing any related log messages.
  pub fn register_service(&self, service: &Service, corpus_path: &str) -> Result<(), Error> {
    services_aggregate::register_service(&self.connection, service, corpus_path)
  }

  /// Extends an existing service on a given corpus (via PATH)
  /// if the service has previously been registered, this call will ignore existing entries and
  /// simply add newly encountered ones
  pub fn extend_service(&self, service: &Service, corpus_path: &str) -> Result<(), Error> {
    services_aggregate::extend_service(&self.connection, service, corpus_path)
  }

  /// Deletes a service by name
  pub fn delete_service_by_name(&self, name: &str) -> Result<usize, Error> {
    services_aggregate::delete_service_by_name(&self.connection, name)
  }

  /// Returns a vector of currently available corpora in the Task store
  pub fn corpora(&self) -> Vec<Corpus> { corpora_aggregate::list_corpora(&self.connection) }

  /// Returns a vector of tasks for a given Corpus, Service and status
  pub fn tasks(&self, corpus: &Corpus, service: &Service, task_status: &TaskStatus) -> Vec<Task> {
    reports::list_tasks(&self.connection, corpus, service, task_status)
  }
  /// Returns a vector of task entry paths for a given Corpus, Service and status
  pub fn entries(
    &self,
    corpus: &Corpus,
    service: &Service,
    task_status: &TaskStatus,
  ) -> Vec<String>
  {
  <<<<<<< loading-info-messages
    use schema::tasks::dsl::{corpus_id, entry, service_id, status};
    let entries: Vec<String> = tasks::table
      .select(entry)
      .filter(service_id.eq(service.id))
      .filter(corpus_id.eq(corpus.id))
      .filter(status.eq(task_status.raw()))
      .load(&self.connection)
      .unwrap_or_default();
    entries
      .into_iter()
      .map(|db_entry_val| {
        let trimmed_entry = db_entry_val.trim_right().to_string();
        if service.name == "import" {
          trimmed_entry
        } else {
          ENTRY_NAME_REGEX.replace(&trimmed_entry, "$1").to_string() + "/" + &service.name + ".zip"
        }
      }).collect()
  }

  /// Provides a progress report, grouped by severity, for a given `Corpus` and `Service` pair
  pub fn progress_report(&self, corpus: &Corpus, service: &Service) -> HashMap<String, f64> {
    use diesel::sql_types::BigInt;
    use schema::tasks::{corpus_id, service_id, status};

    let mut stats_hash: HashMap<String, f64> = HashMap::new();
    for status_key in TaskStatus::keys() {
      stats_hash.insert(status_key, 0.0);
    }
    stats_hash.insert("total".to_string(), 0.0);
    let rows: Vec<(i32, i64)> = tasks::table
      .select((status, sql::<BigInt>("count(*) AS status_count")))
      .filter(service_id.eq(service.id))
      .filter(corpus_id.eq(corpus.id))
      .group_by(tasks::status)
      .order(sql::<BigInt>("status_count").desc())
      .load(&self.connection)
      .unwrap_or_default();
    for &(raw_status, count) in &rows {
      let task_status = TaskStatus::from_raw(raw_status);
      let status_key = task_status.to_key();
      {
        let status_frequency = stats_hash.entry(status_key).or_insert(0.0);
        *status_frequency += count as f64;
      }
      if task_status != TaskStatus::Invalid {
        // DIScount invalids from the total numbers
        let total_frequency = stats_hash.entry("total".to_string()).or_insert(0.0);
        *total_frequency += count as f64;
      }
    }
    Backend::aux_stats_compute_percentages(&mut stats_hash, None);
    stats_hash
  =======
    reports::list_entries(&self.connection, corpus, service, task_status)
  >>>>>>> dependabot/cargo/sys-info-0.9.0
  }

  /// Given a complex selector, of a `Corpus`, `Service`, and the optional `severity`, `category`
  /// and `what`, Provide a progress report at the chosen granularity
  <<<<<<< loading-info-messages
  pub fn task_report(
    &self,
    corpus: &Corpus,
    service: &Service,
    severity_opt: Option<String>,
    category_opt: Option<String>,
    what_opt: Option<String>,
    mut all_messages: bool,
    offset: i64,
    page_size: i64,
  ) -> Vec<HashMap<String, String>>
  {
    use diesel::sql_types::{BigInt, Text};
    use schema::tasks::dsl::{corpus_id, service_id, status};

    // The final report, populated based on the specific selectors
    let mut report = Vec::new();

    if let Some(severity_name) = severity_opt {
      let task_status = TaskStatus::from_key(&severity_name);
      // NoProblem report is a bit special, as it provides a simple list of entries - we assume no
      // logs of notability for this severity.
      if task_status == Some(TaskStatus::NoProblem) {
        let entry_rows: Vec<(String, i64)> = tasks::table
          .select((tasks::entry, tasks::id))
          .filter(service_id.eq(service.id))
          .filter(corpus_id.eq(corpus.id))
          .filter(status.eq(task_status.unwrap().raw()))
          .order(tasks::entry.asc())
          .offset(offset as i64)
          .limit(page_size as i64)
          .load(&self.connection)
          .unwrap_or_default();
        for &(ref entry_fixedwidth, entry_taskid) in &entry_rows {
          let mut entry_map = HashMap::new();
          let entry_trimmed = entry_fixedwidth.trim_right().to_string();
          let entry_name = TASK_REPORT_NAME_REGEX
            .replace(&entry_trimmed, "$1")
            .to_string();

          entry_map.insert("entry".to_string(), entry_trimmed);
          entry_map.insert("entry_name".to_string(), entry_name);
          entry_map.insert("entry_taskid".to_string(), entry_taskid.to_string());
          entry_map.insert("details".to_string(), "OK".to_string());
          report.push(entry_map);
        }
      } else {
        // The "total tasks" used in the divison denominators for computing the percentage
        // distributions are all valid tasks (total - invalid), as we don't want to dilute
        // the service percentage with jobs that were never processed. For now the fastest
        // way to obtain that number is using 2 queries for each and subtracting the numbers in Rust
        let total_count: i64 = tasks::table
          .filter(service_id.eq(service.id))
          .filter(corpus_id.eq(corpus.id))
          .count()
          .get_result(&self.connection)
          .unwrap();
        let invalid_count: i64 = tasks::table
          .filter(service_id.eq(service.id))
          .filter(corpus_id.eq(corpus.id))
          .filter(status.eq(TaskStatus::Invalid.raw()))
          .count()
          .get_result(&self.connection)
          .unwrap();
        let total_valid_count = total_count - invalid_count;

        let log_table = match task_status {
          Some(ref ts) => ts.to_table(),
          None => {
            all_messages = true;
            "log_infos".to_string()
          },
        };

        let task_status_raw = task_status.unwrap_or(TaskStatus::Fatal).raw();
        let status_clause = if !all_messages {
          String::from("status=$3")
        } else {
          String::from("status < $3 and status > ") + &TaskStatus::Invalid.raw().to_string()
        };
        let bind_status = if !all_messages {
          task_status_raw
        } else {
          task_status_raw + 1 // TODO: better would be a .prev() method or so, since this hardwires the assumption of
                              // using adjacent negative integers
        };
        match category_opt {
          None => {
            // Bad news, query is close to line noise
            // Good news, we avoid the boilerplate of dispatching to 4 distinct log tables for now
            let category_report_string =
              "SELECT category as report_name, count(*) as task_count, COALESCE(SUM(total_counts::integer),0) as message_count FROM (".to_string()+
                "SELECT "+&log_table+".category, "+&log_table+".task_id, count(*) as total_counts FROM "+
                  "tasks LEFT OUTER JOIN "+&log_table+" ON (tasks.id="+&log_table+".task_id) WHERE service_id=$1 and corpus_id=$2 and "+ &status_clause +
                    " GROUP BY "+&log_table+".category, "+&log_table+".task_id) as tmp "+
              "GROUP BY category ORDER BY task_count desc";
            let category_report_query = sql_query(category_report_string);
            let category_report_rows: Vec<AggregateReport> = category_report_query
              .bind::<BigInt, i64>(i64::from(service.id))
              .bind::<BigInt, i64>(i64::from(corpus.id))
              .bind::<BigInt, i64>(i64::from(bind_status))
              .load(&self.connection)
              .unwrap_or_default();

            // How many tasks total in this severity-status?
            let severity_tasks: i64 = if !all_messages {
              tasks::table
                .filter(service_id.eq(service.id))
                .filter(corpus_id.eq(corpus.id))
                .filter(status.eq(task_status_raw))
                .count()
                .get_result(&self.connection)
                .unwrap_or(-1)
            } else {
              tasks::table
                .filter(service_id.eq(service.id))
                .filter(corpus_id.eq(corpus.id))
                .count()
                .get_result(&self.connection)
                .unwrap_or(-1)
            };
            let status_report_query_string =
            "SELECT NULL as report_name, count(*) as task_count, COALESCE(SUM(inner_message_count::integer),0) as message_count FROM ( ".to_string()+
              "SELECT tasks.id, count(*) as inner_message_count FROM "+
              "tasks, "+&log_table+" where tasks.id="+&log_table+".task_id and "+
              "service_id=$1 and corpus_id=$2 and "+&status_clause+" group by tasks.id) as tmp";
            let status_report_query = sql_query(status_report_query_string)
              .bind::<BigInt, i64>(i64::from(service.id))
              .bind::<BigInt, i64>(i64::from(corpus.id))
              .bind::<BigInt, i64>(i64::from(bind_status));
            let status_report_rows_result = status_report_query.get_result(&self.connection);
            let status_report_rows: AggregateReport = status_report_rows_result.unwrap();

            let logged_task_count: i64 = status_report_rows.task_count;
            let logged_message_count: i64 = status_report_rows.message_count;
            let silent_task_count = if logged_task_count >= severity_tasks {
              None
            } else {
              Some(severity_tasks - logged_task_count)
            };
            report = Backend::aux_task_rows_stats(
              &category_report_rows,
              total_valid_count,
              severity_tasks,
              logged_message_count,
              silent_task_count,
            )
          },
          Some(category_name) => if category_name == "no_messages" {
            let no_messages_query_string = "SELECT * FROM tasks t WHERE ".to_string()
              + "service_id=$1 and corpus_id=$2 and "
              + &status_clause
              + " and "
              + "NOT EXISTS (SELECT null FROM "
              + &log_table
              + " where "
              + &log_table
              + ".task_id=t.id) limit 100";
            let no_messages_query = sql_query(no_messages_query_string)
              .bind::<BigInt, i64>(i64::from(service.id))
              .bind::<BigInt, i64>(i64::from(corpus.id))
              .bind::<BigInt, i64>(i64::from(bind_status))
              .bind::<BigInt, i64>(i64::from(task_status_raw));
            let no_message_tasks: Vec<Task> = no_messages_query
              .get_results(&self.connection)
              .unwrap_or_default();

            for task in &no_message_tasks {
              let mut entry_map = HashMap::new();
              let entry = task.entry.trim_right().to_string();
              let entry_name = TASK_REPORT_NAME_REGEX.replace(&entry, "$1").to_string();

              entry_map.insert("entry".to_string(), entry);
              entry_map.insert("entry_name".to_string(), entry_name);
              entry_map.insert("entry_taskid".to_string(), task.id.to_string());
              entry_map.insert("details".to_string(), "OK".to_string());
              report.push(entry_map);
            }
          } else {
            match what_opt {
              None => {
                let what_report_query_string =
              "SELECT what as report_name, count(*) as task_count, COALESCE(SUM(total_counts::integer),0) as message_count FROM ( ".to_string() +
                "SELECT "+&log_table+".what, "+&log_table+".task_id, count(*) as total_counts FROM "+
                  "tasks LEFT OUTER JOIN "+&log_table+" ON (tasks.id="+&log_table+".task_id) "+
                  "WHERE service_id=$1 and corpus_id=$2 and "+&status_clause+" and category=$4 "+
                  "GROUP BY "+&log_table+".what, "+&log_table+".task_id) as tmp GROUP BY what ORDER BY task_count desc";
                let what_report_query = sql_query(what_report_query_string)
                  .bind::<BigInt, i64>(i64::from(service.id))
                  .bind::<BigInt, i64>(i64::from(corpus.id))
                  .bind::<BigInt, i64>(i64::from(bind_status))
                  .bind::<Text, _>(category_name.clone());
                let what_report: Vec<AggregateReport> = what_report_query
                  .get_results(&self.connection)
                  .unwrap_or_default();
                // How many tasks and messages total in this category?
                let this_category_report_query_string = "SELECT NULL as report_name, count(*) as task_count, COALESCE(SUM(inner_message_count::integer),0) as message_count FROM".to_string() +
                " (SELECT tasks.id, count(*) as inner_message_count "+
                "FROM tasks, "+&log_table+" WHERE tasks.id="+&log_table+".task_id and "+
                  "service_id=$1 and corpus_id=$2 and "+&status_clause+" and category=$4 group by tasks.id) as tmp";
                let this_category_report_query = sql_query(this_category_report_query_string)
                  .bind::<BigInt, i64>(i64::from(service.id))
                  .bind::<BigInt, i64>(i64::from(corpus.id))
                  .bind::<BigInt, i64>(i64::from(bind_status))
                  .bind::<Text, _>(category_name);
                let this_category_report: AggregateReport = this_category_report_query
                  .get_result(&self.connection)
                  .unwrap();

                report = Backend::aux_task_rows_stats(
                  &what_report,
                  total_valid_count,
                  this_category_report.task_count,
                  this_category_report.message_count,
                  None,
                )
              },
              Some(what_name) => {
                let details_report_query_string = "SELECT tasks.id, tasks.entry, ".to_string()
                  + &log_table
                  + ".details from tasks, "
                  + &log_table
                  + " WHERE tasks.id="
                  + &log_table
                  + ".task_id and service_id=$1 and corpus_id=$2 and "
                  + &status_clause
                  + "and category=$4 and what=$5 ORDER BY tasks.entry ASC offset $6 limit $7";

                let details_report_query = sql_query(details_report_query_string)
                  .bind::<BigInt, i64>(i64::from(service.id))
                  .bind::<BigInt, i64>(i64::from(corpus.id))
                  .bind::<BigInt, i64>(i64::from(bind_status))
                  .bind::<Text, _>(category_name)
                  .bind::<Text, _>(what_name)
                  .bind::<BigInt, i64>(i64::from(offset))
                  .bind::<BigInt, i64>(i64::from(page_size));
                let details_report: Vec<TaskDetailReport> = details_report_query
                  .get_results(&self.connection)
                  .unwrap_or_default();
                for details_row in details_report {
                  let mut entry_map = HashMap::new();
                  let entry = details_row.entry.trim_right().to_string();
                  let entry_name = TASK_REPORT_NAME_REGEX.replace(&entry, "$1").to_string();
                  // TODO: Also use url-escape
                  entry_map.insert("entry".to_string(), entry);
                  entry_map.insert("entry_name".to_string(), entry_name);
                  entry_map.insert("entry_taskid".to_string(), details_row.id.to_string());
                  entry_map.insert("details".to_string(), details_row.details);
                  report.push(entry_map);
                }
              },
            }
          },
        }
      }
    }
    report
  =======
  pub fn task_report(&self, options: TaskReportOptions) -> Vec<HashMap<String, String>> {
    reports::task_report(&self.connection, options)
  >>>>>>> dependabot/cargo/sys-info-0.9.0
  }
  /// Provides a progress report, grouped by severity, for a given `Corpus` and `Service` pair
  pub fn progress_report(&self, corpus: &Corpus, service: &Service) -> HashMap<String, f64> {
    reports::progress_report(&self.connection, corpus.id, service.id)
  }
}
