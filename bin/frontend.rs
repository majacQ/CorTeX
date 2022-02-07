// Copyright 2015-2018 Deyan Ginev. See the LICENSE
// file at the top-level directory of this distribution.
//
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed
// except according to those terms.
#![feature(proc_macro_hygiene, decl_macro)]
#![allow(clippy::implicit_hasher, clippy::let_unit_value)]
#[macro_use]
  <<<<<<< loading-info-messages
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate cortex;
extern crate redis;
extern crate regex;
extern crate time;

use futures::{Future, Stream};
use hyper::header::{ContentLength, ContentType};
use hyper::Client;
use hyper::{Method, Request};
use hyper_tls::HttpsConnector;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::response::status::{Accepted, NotFound};
use rocket::response::{NamedFile, Redirect};
use rocket_contrib::Template;

use tokio_core::reactor::Core;
  =======
extern crate rocket;
  <<<<<<< dependabot/cargo/sys-info-0.9.0
  >>>>>>> dependabot/cargo/sys-info-0.9.0

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::thread;
  =======
extern crate google_signin;
  >>>>>>> dependabot/cargo/redis-0.20.0

use rocket::request::Form;
use rocket::response::status::{Accepted, NotFound};
use rocket::response::{NamedFile, Redirect};
use rocket::Data;
use rocket_contrib::json::Json;
use rocket_contrib::templates::Template;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process;
use std::time::SystemTime;

use cortex::backend::Backend;
use cortex::concerns::CortexInsertable;
use cortex::frontend::concerns::{
  serve_entry, serve_entry_preview, serve_report, serve_rerun, UNKNOWN,
};
use cortex::frontend::cors::CORS;
use cortex::frontend::helpers::*;
use cortex::frontend::params::{AuthParams, ReportParams, RerunRequestParams, TemplateContext};
use cortex::models::{
  Corpus, HistoricalRun, NewCorpus, NewUser, RunMetadata, RunMetadataStack, Service,
  User,
};

  <<<<<<< loading-info-messages
lazy_static! {
  static ref STRIP_NAME_REGEX: Regex = Regex::new(r"/[^/]+$").unwrap();
}

pub struct CORS();

impl Fairing for CORS {
  fn info(&self) -> Info {
    Info {
      name: "Add CORS headers to requests",
      kind: Kind::Response,
    }
  }

  fn on_response(&self, request: &rocket::Request, response: &mut rocket::Response) {
    if request.method() == rocket::http::Method::Options
      || response.content_type() == Some(rocket::http::ContentType::JSON)
    {
      response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
      response.set_header(Header::new(
        "Access-Control-Allow-Methods",
        "POST, GET, OPTIONS",
      ));
      response.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type"));
      response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
      response.set_header(Header::new(
        "Content-Security-Policy-Report-Only",
        "default-src https:; report-uri /csp-violation-report-endpoint/",
      ));
    }

    if request.method() == rocket::http::Method::Options {
      response.set_header(rocket::http::ContentType::Plain);
      response.set_sized_body(Cursor::new(""));
    }
  }
}

static UNKNOWN: &'static str = "_unknown_";

#[derive(Deserialize, Serialize, Debug, Clone)]
struct CortexConfig {
  captcha_secret: String,
  rerun_tokens: HashMap<String, String>,
}

#[derive(Serialize)]
struct TemplateContext {
  global: HashMap<String, String>,
  corpora: Option<Vec<HashMap<String, String>>>,
  services: Option<Vec<HashMap<String, String>>>,
  entries: Option<Vec<HashMap<String, String>>>,
  categories: Option<Vec<HashMap<String, String>>>,
  whats: Option<Vec<HashMap<String, String>>>,
  workers: Option<Vec<HashMap<String, String>>>,
}
impl Default for TemplateContext {
  fn default() -> Self {
    TemplateContext {
      global: HashMap::new(),
      corpora: None,
      services: None,
      entries: None,
      categories: None,
      whats: None,
      workers: None,
    }
  }
}

fn aux_load_config() -> CortexConfig {
  let mut config_file = match File::open("config.json") {
    Ok(cfg) => cfg,
    Err(e) => panic!(
      "You need a well-formed JSON config.json file to run the frontend. Error: {}",
      e
    ),
  };
  let mut config_buffer = String::new();
  match config_file.read_to_string(&mut config_buffer) {
    Ok(_) => {},
    Err(e) => panic!(
      "You need a well-formed JSON config.json file to run the frontend. Error: {}",
      e
    ),
  };

  match serde_json::from_str(&config_buffer) {
    Ok(decoded) => decoded,
    Err(e) => panic!(
      "You need a well-formed JSON config.json file to run the frontend. Error: {}",
      e
    ),
  }
}

#[derive(FromForm)]
struct ReportParams {
  all: Option<bool>,
  offset: Option<i64>,
  page_size: Option<i64>,
}

  =======
  >>>>>>> dependabot/cargo/sys-info-0.9.0
#[get("/")]
fn root() -> Template {
  let mut context = TemplateContext::default();
  let mut global = global_defaults();
  global.insert(
    "title".to_string(),
    "Overview of available Corpora".to_string(),
  );
  global.insert(
    "description".to_string(),
    "An analysis framework for corpora of TeX/LaTeX documents - overview page".to_string(),
  );

  let backend = Backend::default();
  let corpora = backend
    .corpora()
    .iter()
    .map(Corpus::to_hash)
    .collect::<Vec<_>>();

  context.global = global;
  context.corpora = Some(corpora);
  decorate_uri_encodings(&mut context);

  Template::render("overview", context)
}

#[get("/dashboard?<params..>")]
fn admin_dashboard(params: Form<AuthParams>) -> Result<Template, Redirect> {
  // Recommended: Let the crate handle everything for you
  let mut current_user = None;
  let id_info = match verify_oauth(&params.token) {
    None => return Err(Redirect::to("/")),
    Some(id_info) => id_info,
  };
  let display = if let Some(ref name) = id_info.name {
    name.to_owned()
  } else {
    String::new()
  };
  let email = id_info.email.unwrap();
  // TODO: If we ever have too many users, this will be too slow. For now, simple enough.
  let backend = Backend::default();
  let users = backend.users();
  let message = if users.is_empty() {
    let first_admin = NewUser {
      admin: true,
      email: email.to_owned(),
      display,
      first_seen: SystemTime::now(),
      last_seen: SystemTime::now(),
    };
    if backend.add(&first_admin).is_ok() {
      format!("Registered admin user for {:?}", email)
    } else {
      format!("Failed to create user for {:?}", email)
    }
  } else {
    // is this user known?
    if let Ok(u) = User::find_by_email(&email, &backend.connection) {
      let is_admin = if u.admin { "(admin)" } else { "" };
      current_user = Some(u);
      format!("Signed in as {:?} {}", email, is_admin)
    } else {
      let new_viewer = NewUser {
        admin: false,
        email: email.to_owned(),
        display,
        first_seen: SystemTime::now(),
        last_seen: SystemTime::now(),
      };
      if backend.add(&new_viewer).is_ok() {
        format!("Registered viewer-level user for {:?}", email)
      } else {
        format!("Failed to create user for {:?}", email)
      }
    }
  };
  if current_user.is_none() {
    // did we end up registering a new user? If so, look it up
    if let Ok(u) = User::find_by_email(&email, &backend.connection) {
      current_user = Some(u);
    }
  }
  // having a registered user, mark as seen
  if let Some(ref u) = current_user {
    u.touch(&backend.connection).expect("DB ran away");
  }
  let mut global = global_defaults();
  global.insert("message".to_string(), message);
  global.insert("title".to_string(), "Admin Interface".to_string());
  global.insert(
    "description".to_string(),
    "An analysis framework for corpora of TeX/LaTeX documents - admin interface.".to_string(),
  );
  Ok(Template::render(
    "admin",
    dashboard_context(backend, current_user, global),
  ))
}

#[post(
  "/dashboard_task/add_corpus?<params..>",
  format = "application/json",
  data = "<corpus_spec>"
)]
fn dashboard_task_add_corpus(
  params: Form<AuthParams>,
  corpus_spec: Json<NewCorpus>,
) -> Result<Accepted<String>, NotFound<String>>
{
  println!("who: {:?}", params);
  let id_info = match verify_oauth(&params.token) {
    None => return Err(NotFound("could not verify OAuth login".to_owned())),
    Some(id_info) => id_info,
  };
  let backend = Backend::default();
  let user = match User::find_by_email(id_info.email.as_ref().unwrap(), &backend.connection) {
    Ok(u) => u,
    _ => return Err(NotFound("no registered user for your email".to_owned())),
  };
  if user.admin {
    println!("dashboard task data: {:?}", corpus_spec);
    let message = match corpus_spec.create(&backend.connection) {
      Ok(_) => "successfully added corpus to DB",
      Err(_) => "failed to create corpus in DB",
    };
    Ok(Accepted(Some(message.to_owned())))
  } else {
    Err(NotFound(
      "User must be admin to execute dashboard actions".to_owned(),
    ))
  }
}

#[get("/workers/<service_name>")]
fn worker_report(service_name: String) -> Result<Template, NotFound<String>> {
  let backend = Backend::default();
  let service_name = uri_unescape(Some(&service_name)).unwrap_or_else(|| UNKNOWN.to_string());
  if let Ok(service) = Service::find_by_name(&service_name, &backend.connection) {
    let mut global = global_defaults();
    global.insert(
      "title".to_string(),
      format!("Worker report for service {} ", &service_name),
    );
    global.insert(
      "description".to_string(),
      format!(
        "Worker report for service {} as registered by the CorTeX dispatcher",
        &service_name
      ),
    );
    global.insert("corpus_name".to_string(), "all".to_string());
    global.insert("service_name".to_string(), service_name.to_string());
    global.insert(
      "service_description".to_string(),
      service.description.clone(),
    );
    // uri links lead to root, since this is a global overview
    global.insert("corpus_name_uri".to_string(), "../".to_string());
    global.insert("service_name_uri".to_string(), "../".to_string());
    let mut context = TemplateContext {
      global,
      ..TemplateContext::default()
    };
    let workers = service
      .select_workers(&backend.connection)
      .unwrap()
      .into_iter()
      .map(Into::into)
      .collect();
    context.workers = Some(workers);
    Ok(Template::render("workers", context))
  } else {
    Err(NotFound(String::from("no such service")))
  }
}

#[get("/corpus/<corpus_name>")]
fn corpus(corpus_name: String) -> Result<Template, NotFound<String>> {
  let backend = Backend::default();
  let corpus_name = uri_unescape(Some(&corpus_name)).unwrap_or_else(|| UNKNOWN.to_string());
  let corpus_result = Corpus::find_by_name(&corpus_name, &backend.connection);
  if let Ok(corpus) = corpus_result {
    let mut global = global_defaults();
    global.insert(
      "title".to_string(),
      "Registered services for ".to_string() + &corpus_name,
    );
    global.insert(
      "description".to_string(),
      "An analysis framework for corpora of TeX/LaTeX documents - registered services for "
        .to_string()
        + &corpus_name,
    );
    global.insert("corpus_name".to_string(), corpus_name);
    global.insert("corpus_description".to_string(), corpus.description.clone());
    let mut context = TemplateContext {
      global,
      ..TemplateContext::default()
    };

    let services_result = corpus.select_services(&backend.connection);
    if let Ok(backend_services) = services_result {
      let services = backend_services
        .iter()
        .map(Service::to_hash)
        .collect::<Vec<_>>();
      let mut service_reports = Vec::new();
      for service in services {
        // TODO: Report on the service status when we improve on the service report UX
        // service.insert("status".to_string(), "Running".to_string());
        service_reports.push(service);
      }
      context.services = Some(service_reports);
    }
    decorate_uri_encodings(&mut context);
    return Ok(Template::render("services", context));
  }
  Err(NotFound(format!(
    "Corpus {} is not registered",
    &corpus_name
  )))
}

#[get("/corpus/<corpus_name>/<service_name>")]
fn top_service_report(
  corpus_name: String,
  service_name: String,
) -> Result<Template, NotFound<String>>
{
  serve_report(corpus_name, service_name, None, None, None, None)
}
#[get("/corpus/<corpus_name>/<service_name>/<severity>")]
fn severity_service_report(
  corpus_name: String,
  service_name: String,
  severity: String,
) -> Result<Template, NotFound<String>>
{
  serve_report(corpus_name, service_name, Some(severity), None, None, None)
}
#[get("/corpus/<corpus_name>/<service_name>/<severity>?<params..>")]
fn severity_service_report_all(
  corpus_name: String,
  service_name: String,
  severity: String,
  params: Option<Form<ReportParams>>,
) -> Result<Template, NotFound<String>>
{
  serve_report(
    corpus_name,
    service_name,
    Some(severity),
    None,
    None,
    params,
  )
}
#[get("/corpus/<corpus_name>/<service_name>/<severity>/<category>")]
fn category_service_report(
  corpus_name: String,
  service_name: String,
  severity: String,
  category: String,
) -> Result<Template, NotFound<String>>
{
  serve_report(
    corpus_name,
    service_name,
    Some(severity),
    Some(category),
    None,
    None,
  )
}
#[get("/corpus/<corpus_name>/<service_name>/<severity>/<category>?<params..>")]
fn category_service_report_all(
  corpus_name: String,
  service_name: String,
  severity: String,
  category: String,
  params: Option<Form<ReportParams>>,
) -> Result<Template, NotFound<String>>
{
  serve_report(
    corpus_name,
    service_name,
    Some(severity),
    Some(category),
    None,
    params,
  )
}

#[get("/corpus/<corpus_name>/<service_name>/<severity>/<category>/<what>")]
fn what_service_report(
  corpus_name: String,
  service_name: String,
  severity: String,
  category: String,
  what: String,
) -> Result<Template, NotFound<String>>
{
  serve_report(
    corpus_name,
    service_name,
    Some(severity),
    Some(category),
    Some(what),
    None,
  )
}
#[get("/corpus/<corpus_name>/<service_name>/<severity>/<category>/<what>?<params..>")]
fn what_service_report_all(
  corpus_name: String,
  service_name: String,
  severity: String,
  category: String,
  what: String,
  params: Option<Form<ReportParams>>,
) -> Result<Template, NotFound<String>>
{
  serve_report(
    corpus_name,
    service_name,
    Some(severity),
    Some(category),
    Some(what),
    params,
  )
}

#[get("/history/<corpus_name>/<service_name>")]
fn historical_runs(
  corpus_name: String,
  service_name: String,
) -> Result<Template, NotFound<String>>
{
  let mut context = TemplateContext::default();
  let mut global = global_defaults();
  let backend = Backend::default();
  let corpus_name = corpus_name.to_lowercase();
  if let Ok(corpus) = Corpus::find_by_name(&corpus_name, &backend.connection) {
    if let Ok(service) = Service::find_by_name(&service_name, &backend.connection) {
      if let Ok(runs) = HistoricalRun::find_by(&corpus, &service, &backend.connection) {
        let runs_meta = runs
          .into_iter()
          .map(Into::into)
          .collect::<Vec<RunMetadata>>();
        let runs_meta_stack: Vec<RunMetadataStack> = RunMetadataStack::transform(&runs_meta);
        context.history_serialized = Some(serde_json::to_string(&runs_meta_stack).unwrap());
        global.insert(
          "history_length".to_string(),
          runs_meta
            .iter()
            .filter(|run| !run.end_time.is_empty())
            .count()
            .to_string(),
        );
        context.history = Some(runs_meta);
      }
    }
  }

  // Pass the globals(reports+metadata) onto the stash
  global.insert(
    "description".to_string(),
    format!(
      "Historical runs of service {} over corpus {}",
      service_name, corpus_name
    ),
  );
  global.insert("service_name".to_string(), service_name);
  global.insert("corpus_name".to_string(), corpus_name);

  context.global = global;
  // And pass the handy lambdas
  // And render the correct template
  decorate_uri_encodings(&mut context);

  // Report also the query times
  Ok(Template::render("history", context))
}

#[get("/preview/<corpus_name>/<service_name>/<entry_name>")]
fn preview_entry(
  corpus_name: String,
  service_name: String,
  entry_name: String,
) -> Result<Template, NotFound<String>>
{
  serve_entry_preview(corpus_name, service_name, entry_name)
}

#[post("/entry/<service_name>/<entry_id>", data = "<data>")]
fn entry_fetch(
  service_name: String,
  entry_id: usize,
  data: Data,
) -> Result<NamedFile, NotFound<String>>
{
  <<<<<<< loading-info-messages
  // Any secrets reside in config.json
  let cortex_config = aux_load_config();

  let g_recaptcha_response_string = if data.len() > 21 {
    str::from_utf8(&data[21..])
      .unwrap_or(UNKNOWN)
      .replace("&g-recaptcha-response=", "")
  } else {
    UNKNOWN.to_owned()
  };
  let g_recaptcha_response = &g_recaptcha_response_string;
  // Check if we hve the g_recaptcha_response in Redis, then reuse
  let redis_opt;
  let quota: usize = match redis::Client::open("redis://127.0.0.1/") {
    Err(_) => {return Err(Redirect::to("/"))}// TODO: Err(NotFound(format!("redis unreachable")))},
    Ok(redis_client) => match redis_client.get_connection() {
      Err(_) => {return Err(Redirect::to("/"))}//TODO: Err(NotFound(format!("redis unreachable")))},
      Ok(redis_connection) => {
        let quota = redis_connection.get(g_recaptcha_response).unwrap_or(0);
        redis_opt = Some(redis_connection);
        quota
      }
    }
  };

  println!("Response: {:?}", g_recaptcha_response);
  println!("Quota: {:?}", quota);
  let captcha_verified = if quota > 0 {
    if let Some(ref redis_connection) = redis_opt {
      println!("Using local redis quota.");
      if quota == 1 {
        // Remove if last
        redis_connection.del(g_recaptcha_response).unwrap_or(());
      } else {
        // We have quota available, decrement it
        redis_connection
          .set(g_recaptcha_response, quota - 1)
          .unwrap_or(());
      }
      // And allow operation
      true
    } else {
      false // no redis, no access.
    }
  } else {
    // expired quota, check with google
    let check_val = aux_check_captcha(g_recaptcha_response, &cortex_config.captcha_secret);
    println!("Google validity: {:?}", check_val);
    if check_val {
      if let Some(ref redis_connection) = redis_opt {
        // Add a reuse quota if things check out, 19 more downloads
        redis_connection.set(g_recaptcha_response, 19).unwrap_or(());
      }
    }
    check_val
  };
  println!("Captcha validity: {:?}", captcha_verified);

  // If you are not human, you have no business here.
  if !captcha_verified {
    if g_recaptcha_response != UNKNOWN {
      return Err(Redirect::to("/expire_captcha"));
    } else {
      return Err(Redirect::to("/"));
    }
  }

  let backend = Backend::default();
  let task = Task::find(entry_id as i64, &backend.connection).unwrap();

  let entry = task.entry;
  let zip_path = match service_name.as_str() {
    "import" => entry,
    _ => STRIP_NAME_REGEX.replace(&entry, "").to_string() + "/" + &service_name + ".zip",
  };
  if zip_path.is_empty() {
    Err(Redirect::to("/")) // TODO : Err(NotFound(format!("Service {:?} does not have a result for entry {:?}",
                           // service_name, entry_id)))
  } else {
    NamedFile::open(&zip_path).map_err(|_| Redirect::to("/"))
  }
  =======
  serve_entry(service_name, entry_id, data)
  >>>>>>> dependabot/cargo/sys-info-0.9.0
}

//Expire captchas
#[get("/expire_captcha")]
fn expire_captcha() -> Result<Template, NotFound<String>> {
  let mut context = TemplateContext::default();
  let mut global = global_defaults();
  global.insert(
    "description".to_string(),
    "Expire captcha cache for CorTeX.".to_string(),
  );
  context.global = global;
  Ok(Template::render("expire_captcha", context))
}

#[post(
  "/rerun/<corpus_name>/<service_name>",
  format = "application/json",
  data = "<rr>"
)]
fn rerun_corpus(
  corpus_name: String,
  service_name: String,
  rr: Json<RerunRequestParams>,
) -> Result<Accepted<String>, NotFound<String>>
{
  let corpus_name = corpus_name.to_lowercase();
  serve_rerun(corpus_name, service_name, None, None, None, rr)
}

#[post(
  "/rerun/<corpus_name>/<service_name>/<severity>",
  format = "application/json",
  data = "<rr>"
)]
fn rerun_severity(
  corpus_name: String,
  service_name: String,
  severity: String,
  rr: Json<RerunRequestParams>,
) -> Result<Accepted<String>, NotFound<String>>
{
  serve_rerun(corpus_name, service_name, Some(severity), None, None, rr)
}

#[post(
  "/rerun/<corpus_name>/<service_name>/<severity>/<category>",
  format = "application/json",
  data = "<rr>"
)]
fn rerun_category(
  corpus_name: String,
  service_name: String,
  severity: String,
  category: String,
  rr: Json<RerunRequestParams>,
) -> Result<Accepted<String>, NotFound<String>>
{
  serve_rerun(
    corpus_name,
    service_name,
    Some(severity),
    Some(category),
    None,
    rr,
  )
}

#[post(
  "/rerun/<corpus_name>/<service_name>/<severity>/<category>/<what>",
  format = "application/json",
  data = "<rr>"
)]
fn rerun_what(
  corpus_name: String,
  service_name: String,
  severity: String,
  category: String,
  what: String,
  rr: Json<RerunRequestParams>,
) -> Result<Accepted<String>, NotFound<String>>
{
  serve_rerun(
    corpus_name,
    service_name,
    Some(severity),
    Some(category),
    Some(what),
    rr,
  )
}

#[get("/favicon.ico")]
fn favicon() -> Result<NamedFile, NotFound<String>> {
  let path = Path::new("public/").join("favicon.ico");
  NamedFile::open(&path).map_err(|_| NotFound(format!("Bad path: {:?}", path)))
}

#[get("/robots.txt")]
fn robots() -> Result<NamedFile, NotFound<String>> {
  let path = Path::new("public/").join("robots.txt");
  NamedFile::open(&path).map_err(|_| NotFound(format!("Bad path: {:?}", path)))
}

#[get("/public/<file..>")]
fn files(file: PathBuf) -> Result<NamedFile, NotFound<String>> {
  let path = Path::new("public/").join(file);
  NamedFile::open(&path).map_err(|_| NotFound(format!("Bad path: {:?}", path)))
}

fn rocket() -> rocket::Rocket {
  rocket::ignite()
    .mount(
      "/",
      routes![
        root,
        admin_dashboard,
        dashboard_task_add_corpus,
        corpus,
        favicon,
        robots,
        files,
        worker_report,
        top_service_report,
        severity_service_report,
        category_service_report,
        what_service_report,
        severity_service_report_all,
        category_service_report_all,
        what_service_report_all,
        preview_entry,
        entry_fetch,
        rerun_corpus,
        rerun_severity,
        rerun_category,
        rerun_what,
        expire_captcha,
        historical_runs
      ],
    )
    .attach(Template::fairing())
    .attach(CORS())
}

fn main() -> Result<(), Box<dyn Error>> {
  let backend = Backend::default();
  // Ensure all cortex daemon services are running in parallel before we sping up the frontend
  // Redis cache expiration logic, for report pages
  let cw_opt = backend
    .ensure_daemon("cache_worker")
    .expect("Couldn't spin up cache worker");
  // Corpus registration init worker, should run on the machine storing the data (currently same as frontend machine)
  let initw_opt = backend
    .ensure_daemon("init_worker")
    .expect("Couldn't spin up init worker");

  // Dispatcher manager, for service execution logic
  let dispatcher_opt = backend
    .ensure_daemon("dispatcher")
    .expect("Couldn't spin up dispatcher");
  backend
    .override_daemon_record("frontend".to_owned(), process::id())
    .expect("Could not register the process id with the backend, aborting...");

  // Finally, start up the web service
  let rocket_error = rocket().launch();
  // If we failed to boot / exited dirty, destroy the children
  if let Some(mut cw) = cw_opt {
    cw.kill()?;
    cw.wait()?;
  }
  if let Some(mut iw) = initw_opt {
    iw.kill()?;
    iw.wait()?;
  }
  if let Some(mut dispatcher) = dispatcher_opt {
    dispatcher.kill()?;
    dispatcher.wait()?;
  }
  drop(rocket_error);
  Ok(())
}
  <<<<<<< loading-info-messages

fn serve_report(
  corpus_name: &str,
  service_name: &str,
  severity: Option<String>,
  category: Option<String>,
  what: Option<String>,
  params: Option<ReportParams>,
) -> Result<Template, NotFound<String>>
{
  let report_start = time::get_time();
  let mut context = TemplateContext::default();
  let mut global = HashMap::new();
  let backend = Backend::default();

  let corpus_result = Corpus::find_by_name(corpus_name, &backend.connection);
  if let Ok(corpus) = corpus_result {
    let service_result = Service::find_by_name(service_name, &backend.connection);
    if let Ok(service) = service_result {
      // Metadata in all reports
      global.insert(
        "title".to_string(),
        "Corpus Report for ".to_string() + corpus_name,
      );
      global.insert(
        "description".to_string(),
        "An analysis framework for corpora of TeX/LaTeX documents - statistical reports for "
          .to_string()
          + corpus_name,
      );
      global.insert("corpus_name".to_string(), corpus_name.to_string());
      global.insert("corpus_description".to_string(), corpus.description.clone());
      global.insert("service_name".to_string(), service_name.to_string());
      global.insert(
        "service_description".to_string(),
        service.description.clone(),
      );
      global.insert("type".to_string(), "Conversion".to_string());
      global.insert("inputformat".to_string(), service.inputformat.clone());
      global.insert("outputformat".to_string(), service.outputformat.clone());

      let all_messages = match params {
        None => false,
        Some(ref params) => *params.all.as_ref().unwrap_or(&false),
      };
      global.insert("all_messages".to_string(), all_messages.to_string());
      if all_messages {
        // Handlebars has a weird limitation on its #if conditional, can only test for field
        // presence. So...
        global.insert("all_messages_true".to_string(), all_messages.to_string());
      }
      match service.inputconverter {
        Some(ref ic_service_name) => {
          global.insert("inputconverter".to_string(), ic_service_name.clone())
        },
        None => global.insert("inputconverter".to_string(), "missing?".to_string()),
      };

      let report;
      let template;
      if severity.is_none() {
        // Top-level report
        report = backend.progress_report(&corpus, &service);
        // Record the report into the globals
        for (key, val) in report {
          global.insert(key.clone(), val.to_string());
        }
        global.insert("report_time".to_string(), time::now().rfc822().to_string());
        template = "report";
      } else if category.is_none() {
        // Severity-level report
        global.insert("severity".to_string(), severity.clone().unwrap());
        global.insert(
          "highlight".to_string(),
          aux_severity_highlight(&severity.clone().unwrap()).to_string(),
        );
        template = if severity.is_some() && (severity.as_ref().unwrap() == "no_problem") {
          let entries = aux_task_report(
            &mut global,
            &corpus,
            &service,
            severity,
            None,
            None,
            &params,
          );
          // Record the report into "entries" vector
          context.entries = Some(entries);
          // And set the task list template
          "task-list-report"
        } else {
          let categories = aux_task_report(
            &mut global,
            &corpus,
            &service,
            severity,
            None,
            None,
            &params,
          );
          // Record the report into "categories" vector
          context.categories = Some(categories);
          // And set the severity template
          "severity-report"
        };
      } else if what.is_none() {
        // Category-level report
        global.insert("severity".to_string(), severity.clone().unwrap());
        global.insert(
          "highlight".to_string(),
          aux_severity_highlight(&severity.clone().unwrap()).to_string(),
        );
        global.insert("category".to_string(), category.clone().unwrap());
        if category.is_some() && (category.as_ref().unwrap() == "no_messages") {
          let entries = aux_task_report(
            &mut global,
            &corpus,
            &service,
            severity,
            category,
            None,
            &params,
          );
          // Record the report into "entries" vector
          context.entries = Some(entries);
          // And set the task list template
          template = "task-list-report";
        } else {
          let whats = aux_task_report(
            &mut global,
            &corpus,
            &service,
            severity,
            category,
            None,
            &params,
          );
          // Record the report into "whats" vector
          context.whats = Some(whats);
          // And set the category template
          template = "category-report";
        }
      } else {
        // What-level report
        global.insert("severity".to_string(), severity.clone().unwrap());
        global.insert(
          "highlight".to_string(),
          aux_severity_highlight(&severity.clone().unwrap()).to_string(),
        );
        global.insert("category".to_string(), category.clone().unwrap());
        global.insert("what".to_string(), what.clone().unwrap());
        let entries = aux_task_report(
          &mut global,
          &corpus,
          &service,
          severity,
          category,
          what,
          &params,
        );
        // Record the report into "entries" vector
        context.entries = Some(entries);
        // And set the task list template
        template = "task-list-report";
      }
      // Pass the globals(reports+metadata) onto the stash
      context.global = global;
      // And pass the handy lambdas
      // And render the correct template
      aux_decorate_uri_encodings(&mut context);

      // Report also the query times
      let report_end = time::get_time();
      let report_duration = (report_end - report_start).num_milliseconds();
      context
        .global
        .insert("report_duration".to_string(), report_duration.to_string());
      Ok(Template::render(template, context))
    } else {
      Err(NotFound(format!(
        "Service {} does not exist.",
        &service_name
      )))
    }
  } else {
    Err(NotFound(format!("Corpus {} does not exist.", &corpus_name)))
  }
}

fn serve_rerun(
  corpus_name: &str,
  service_name: &str,
  severity: Option<String>,
  category: Option<String>,
  what: Option<String>,
  token_bytes: &[u8],
) -> Result<Accepted<String>, NotFound<String>>
{
  let config = aux_load_config();
  // let corpus_name =
  // aux_uri_unescape(request.param("corpus_name")).unwrap_or(UNKNOWN.to_string());
  // let service_name =
  // aux_uri_unescape(request.param("service_name")).unwrap_or(UNKNOWN.to_string()); let severity
  // = aux_uri_unescape(request.param("severity")); let category =
  // aux_uri_unescape(request.param("category")); let what =
  // aux_uri_unescape(request.param("what"));

  // Ensure we're given a valid rerun token to rerun, or anyone can wipe the cortex results
  let token = str::from_utf8(token_bytes).unwrap_or(UNKNOWN);
  let user_opt = config.rerun_tokens.get(token);
  let user = match user_opt {
    None => return Err(NotFound("Access Denied".to_string())), /* TODO: response.error(Forbidden,
                                                              * "Access denied"), */
    Some(user) => user,
  };
  println!(
    "-- User {:?}: Mark for rerun on {:?}/{:?}/{:?}/{:?}/{:?}",
    user, corpus_name, service_name, severity, category, what
  );

  // Run (and measure) the three rerun queries
  let report_start = time::get_time();
  let backend = Backend::default();
  // Build corpus and service objects
  let corpus = match Corpus::find_by_name(corpus_name, &backend.connection) {
    Err(_) => return Err(NotFound("Access Denied".to_string())), /* TODO: response.error(Forbidden,
                                                                * "Access denied"), */
    Ok(corpus) => corpus,
  };

  let service = match Service::find_by_name(service_name, &backend.connection) {
    Err(_) => return Err(NotFound("Access Denied".to_string())), /* TODO: response.error(Forbidden,
                                                                * "Access denied"), */
    Ok(service) => service,
  };
  let rerun_result = backend.mark_rerun(&corpus, &service, severity, category, what);
  let report_end = time::get_time();
  let report_duration = (report_end - report_start).num_milliseconds();
  println!(
    "-- User {:?}: Mark for rerun took {:?}ms",
    user, report_duration
  );
  match rerun_result {
    Err(_) => Err(NotFound("Access Denied".to_string())), // TODO: better error message?
    Ok(_) => Ok(Accepted(None)),
  }
}

fn aux_severity_highlight(severity: &str) -> &str {
  match severity {
    // Bootstrap highlight classes
    "no_problem" => "success",
    "warning" => "warning",
    "error" => "error",
    "fatal" => "danger",
    "invalid" => "info",
    _ => "unknown",
  }
}
fn aux_uri_unescape(param: Option<&str>) -> Option<String> {
  match param {
    None => None,
    Some(param_encoded) => {
      let mut param_decoded: String = param_encoded.to_owned();
      // TODO: This could/should be done faster by using lazy_static!
      for &(original, replacement) in &[
        ("%3A", ":"),
        ("%2F", "/"),
        ("%24", "$"),
        ("%2E", "."),
        ("%21", "!"),
        ("%40", "@"),
      ] {
        param_decoded = param_decoded.replace(original, replacement);
      }
      Some(
        url::percent_encoding::percent_decode(param_decoded.as_bytes())
          .decode_utf8_lossy()
          .into_owned(),
      )
    },
  }
}
fn aux_uri_escape(param: Option<String>) -> Option<String> {
  match param {
    None => None,
    Some(param_pure) => {
      let mut param_encoded: String = url::percent_encoding::utf8_percent_encode(
        &param_pure,
        url::percent_encoding::DEFAULT_ENCODE_SET,
      ).collect::<String>();
      // TODO: This could/should be done faster by using lazy_static!
      for &(original, replacement) in &[
        (":", "%3A"),
        ("/", "%2F"),
        ("\\", "%5C"),
        ("$", "%24"),
        (".", "%2E"),
        ("!", "%21"),
        ("@", "%40"),
      ] {
        param_encoded = param_encoded.replace(original, replacement);
      }
      // if param_pure != param_encoded {
      //   println!("Encoded {:?} to {:?}", param_pure, param_encoded);
      // } else {
      //   println!("No encoding needed: {:?}", param_pure);
      // }
      Some(param_encoded)
    },
  }
}
fn aux_decorate_uri_encodings(context: &mut TemplateContext) {
  for inner_vec in &mut [
    &mut context.corpora,
    &mut context.services,
    &mut context.entries,
    &mut context.categories,
    &mut context.whats,
  ] {
    if let Some(ref mut inner_vec_data) = **inner_vec {
      for mut subhash in inner_vec_data {
        let mut uri_decorations = vec![];
        for (subkey, subval) in subhash.iter() {
          uri_decorations.push((
            subkey.to_string() + "_uri",
            aux_uri_escape(Some(subval.to_string())).unwrap(),
          ));
        }
        for (decoration_key, decoration_val) in uri_decorations {
          subhash.insert(decoration_key, decoration_val);
        }
      }
    }
  }
  // global is handled separately
  let mut uri_decorations = vec![];
  for (subkey, subval) in &context.global {
    uri_decorations.push((
      subkey.to_string() + "_uri",
      aux_uri_escape(Some(subval.to_string())).unwrap(),
    ));
  }
  for (decoration_key, decoration_val) in uri_decorations {
    context.global.insert(decoration_key, decoration_val);
  }
  let mut current_link = String::new();
  {
    if let Some(corpus_name) = context.global.get("corpus_name_uri") {
      if let Some(service_name) = context.global.get("service_name_uri") {
        current_link = format!("/corpus/{}/{}/", corpus_name, service_name);
        if let Some(severity) = context.global.get("severity_uri") {
          current_link.push_str(severity);
          current_link.push('/');
          if let Some(category) = context.global.get("category_uri") {
            current_link.push_str(category);
            current_link.push('/');
            if let Some(what) = context.global.get("what_uri") {
              current_link.push_str(what);
            }
          }
        }
      }
    }
  }
  if !current_link.is_empty() {
    context
      .global
      .insert("current_link_uri".to_string(), current_link);
  }
}

#[derive(Deserialize)]
struct IsSuccess {
  success: bool,
}

fn aux_check_captcha(g_recaptcha_response: &str, captcha_secret: &str) -> bool {
  let mut core = match Core::new() {
    Ok(c) => c,
    _ => return false,
  };
  let handle = core.handle();
  let client = Client::configure()
    .connector(HttpsConnector::new(4, &handle).unwrap())
    .build(&handle);

  let mut verified = false;
  let url_with_query = "https://www.google.com/recaptcha/api/siteverify?secret=".to_string()
    + captcha_secret
    + "&response="
    + g_recaptcha_response;
  let json_str = format!(
    "{{\"secret\":\"{:?}\",\"response\":\"{:?}\"}}",
    captcha_secret, g_recaptcha_response
  );
  let req_url = match url_with_query.parse() {
    Ok(parsed) => parsed,
    _ => return false,
  };
  let mut req = Request::new(Method::Post, req_url);
  req.headers_mut().set(ContentType::json());
  req.headers_mut().set(ContentLength(json_str.len() as u64));
  req.set_body(json_str);

  let post = client.request(req).and_then(|res| res.body().concat2());
  let posted = match core.run(post) {
    Ok(posted_data) => match str::from_utf8(&posted_data) {
      Ok(posted_str) => posted_str.to_string(),
      Err(e) => {
        println!("err: {}", e);
        return false;
      },
    },
    Err(e) => {
      println!("err: {}", e);
      return false;
    },
  };
  let json_decoded: Result<IsSuccess, _> = serde_json::from_str(&posted);
  if let Ok(response_json) = json_decoded {
    if response_json.success {
      verified = true;
    }
  }

  verified
}

fn aux_task_report(
  global: &mut HashMap<String, String>,
  corpus: &Corpus,
  service: &Service,
  severity: Option<String>,
  category: Option<String>,
  what: Option<String>,
  params: &Option<ReportParams>,
) -> Vec<HashMap<String, String>>
{
  let all_messages = match params {
    None => false,
    Some(ref params) => *params.all.as_ref().unwrap_or(&false),
  };
  let offset = match params {
    None => 0,
    Some(ref params) => *params.offset.as_ref().unwrap_or(&0),
  };
  let page_size = match params {
    None => 100,
    Some(ref params) => *params.page_size.as_ref().unwrap_or(&100),
  };
  let fetched_report;
  let mut time_val: String = time::now().rfc822().to_string();

  let mut redis_connection = match redis::Client::open("redis://127.0.0.1/") {
    Ok(redis_client) => match redis_client.get_connection() {
      Ok(rc) => Some(rc),
      _ => None,
    },
    _ => None,
  };

  let mut cache_key = String::new();
  let mut cache_key_time = String::new();
  let cached_report: Vec<HashMap<String, String>> =
    if what.is_some() || severity == Some("no_problem".to_string()) {
      vec![]
    } else {
      // Levels 1-3 get cached, except no_problem pages
      let key_tail = match severity.clone() {
        Some(severity) => {
          let cat_tail = match category.clone() {
            Some(category) => {
              let what_tail = match what.clone() {
                Some(what) => "_".to_string() + &what,
                None => String::new(),
              };
              "_".to_string() + &category + &what_tail
            },
            None => String::new(),
          };
          "_".to_string() + &severity + &cat_tail
        },
        None => String::new(),
      } + if all_messages { "_all_messages" } else { "" };
      cache_key = corpus.id.to_string() + "_" + &service.id.to_string() + &key_tail;
      cache_key_time = cache_key.clone() + "_time";
      let cache_val: String = if let Some(ref rc) = redis_connection {
        rc.get(cache_key.clone()).unwrap_or_default()
      } else {
        String::new()
      };
      if cache_val.is_empty() {
        vec![]
      } else {
        serde_json::from_str(&cache_val).unwrap_or_default()
      }
    };

  if cached_report.is_empty() {
    let backend = Backend::default();
    fetched_report = backend.task_report(
      corpus,
      service,
      severity.clone(),
      category,
      what.clone(),
      all_messages,
      offset,
      page_size,
    );
    if what.is_none() && severity != Some("no_problem".to_string()) {
      let report_json: String = serde_json::to_string(&fetched_report).unwrap();
      // don't cache the task list pages

      if let Some(ref mut rc) = redis_connection {
        let _: () = rc.set(cache_key, report_json).unwrap();
      }

      if let Some(ref mut rc) = redis_connection {
        let _: () = rc.set(cache_key_time, time_val.clone()).unwrap();
      }
    }
  } else {
    // Get the report time, so that the user knows where the data is coming from
    time_val = if let Some(ref mut rc) = redis_connection {
      match rc.get(cache_key_time) {
        Ok(tval) => tval,
        Err(_) => time::now().rfc822().to_string(),
      }
    } else {
      time::now().rfc822().to_string()
    };
    fetched_report = cached_report;
  }

  // Setup the return

  let from_offset = offset;
  let to_offset = offset + page_size;
  global.insert("from_offset".to_string(), from_offset.to_string());
  if from_offset >= page_size {
    // TODO: properly do tera ifs?
    global.insert("offset_min_false".to_string(), "true".to_string());
    global.insert(
      "prev_offset".to_string(),
      (from_offset - page_size).to_string(),
    );
  }

  if fetched_report.len() >= page_size as usize {
    global.insert("offset_max_false".to_string(), "true".to_string());
  }
  global.insert(
    "next_offset".to_string(),
    (from_offset + page_size).to_string(),
  );

  global.insert("offset".to_string(), offset.to_string());
  global.insert("page_size".to_string(), page_size.to_string());
  global.insert("to_offset".to_string(), to_offset.to_string());
  global.insert("report_time".to_string(), time_val);

  fetched_report
}

fn cache_worker() {
  let redis_client = match redis::Client::open("redis://127.0.0.1/") {
    Ok(client) => client,
    _ => panic!("Redis connection failed, please boot up redis and restart the frontend!"),
  };
  let redis_connection = match redis_client.get_connection() {
    Ok(conn) => conn,
    _ => panic!("Redis connection failed, please boot up redis and restart the frontend!"),
  };
  let mut queued_cache: HashMap<String, usize> = HashMap::new();
  loop {
    // Keep a fresh backend connection on each invalidation pass.
    let backend = Backend::default();
    let mut global_stub: HashMap<String, String> = HashMap::new();
    // each corpus+service (non-import)
    for corpus in &backend.corpora() {
      if let Ok(services) = corpus.select_services(&backend.connection) {
        for service in &services {
          if service.name == "import" {
            continue;
          }
          println!(
            "[cache worker] Examining corpus {:?}, service {:?}",
            corpus.name, service.name
          );
          // Pages we'll cache:
          let report = backend.progress_report(corpus, service);
          let zero: f64 = 0.0;
          let huge: usize = 999_999;
          let queued_count_f64: f64 =
            report.get("queued").unwrap_or(&zero) + report.get("todo").unwrap_or(&zero);
          let queued_count: usize = queued_count_f64 as usize;
          let key_base: String = corpus.id.to_string() + "_" + &service.id.to_string();
          // Only recompute the inner pages if we are seeing a change / first visit, on the top
          // corpus+service level
          if *queued_cache.get(&key_base).unwrap_or(&huge) != queued_count {
            println!("[cache worker] state changed, invalidating ...");
            // first cache the count for the next check:
            queued_cache.insert(key_base.clone(), queued_count);
            // each reported severity (fatal, warning, error)
            for severity in &["invalid", "fatal", "error", "warning", "no_problem", "info"] {
              // most importantly, DEL the key from Redis!
              let key_severity = key_base.clone() + "_" + severity;
              println!("[cache worker] DEL {:?}", key_severity);
              redis_connection.del(key_severity.clone()).unwrap_or(());
              // also the combined-severity page for this category
              let key_severity_all = key_severity.clone() + "_all_messages";
              println!("[cache worker] DEL {:?}", key_severity_all);
              redis_connection.del(key_severity_all.clone()).unwrap_or(());
              if "no_problem" == *severity {
                continue;
              }

              // cache category page
              thread::sleep(Duration::new(1, 0)); // Courtesy sleep of 1 second.
              let category_report = aux_task_report(
                &mut global_stub,
                corpus,
                service,
                Some(severity.to_string()),
                None,
                None,
                &None,
              );
              // for each category, cache the what page
              for cat_hash in &category_report {
                let string_empty = String::new();
                let category = cat_hash.get("name").unwrap_or(&string_empty);
                if category.is_empty() || (category == "total") {
                  continue;
                }

                let key_category = key_severity.clone() + "_" + category;
                println!("[cache worker] DEL {:?}", key_category);
                redis_connection.del(key_category.clone()).unwrap_or(());
                // also the combined-severity page for this `what` class
                let key_category_all = key_category + "_all_messages";
                println!("[cache worker] DEL {:?}", key_category_all);
                redis_connection.del(key_category_all.clone()).unwrap_or(());

                let _ = aux_task_report(
                  &mut global_stub,
                  corpus,
                  service,
                  Some(severity.to_string()),
                  Some(category.to_string()),
                  None,
                  &None,
                );
              }
            }
          }
        }
      }
    }
    // Take two minutes before we recheck.
    thread::sleep(Duration::new(120, 0));
  }
}
  =======
  >>>>>>> dependabot/cargo/sys-info-0.9.0
