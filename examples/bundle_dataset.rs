// Copyright 2015-2016 Deyan Ginev. See the LICENSE
// file at the top-level directory of this distribution.
//
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed
// except according to those terms.

extern crate cortex;
extern crate time;
extern crate libxml;
extern crate Archive;
extern crate regex;

use std::env;
use std::str;
use regex::Regex;
use Archive::*;
use libxml::parser::Parser;
use cortex::backend::Backend;
use cortex::models::{Corpus, Service};
use cortex::helpers::TaskStatus;

/// Extends all corpora registered with the `CorTeX` backend, with any new available sources
///  (example usage: arXiv.org releases new source bundles every month, which warrant an update at the same frequency.)
fn main() {
  let start_bundle = time::get_time();
  // Setup CorTeX backend data
  let backend = Backend::default();
  let mut input_args = env::args();
  let _ = input_args.next(); // discard the script filename
  let corpus_name = match input_args.next() {
    Some(name) => name,
    None => "arXMLiv".to_string(),
  };
  let service_name = match input_args.next() {
    Some(name) => name,
    None => "tex_to_html".to_string(),
  };
  let dataset_path = match input_args.next() {
    Some(path) => path,
    None => "./dataset.zip".to_string(),
  };
  let corpus = match Corpus::find_by_name(&corpus_name, &backend.connection) {
    Ok(c) => c,
    Err(e) => {
      println!("Failed to load corpus: {:?}", e);
      return;
    }
  };
  let service = match Service::find_by_name(&service_name, &backend.connection) {
    Ok(s) => s,
    Err(e) => {
      println!("Failed to load service: {:?}", e);
      return;
    }
  };
  // Setup document parser
  let parser = Parser::default_html();
  // Set the extension regex
  let entry_name_regex = Regex::new(r"([^/]+)/([^/]+)/([^/]+)\.zip$").unwrap();
  // Set the database archive file
  let mut total_dataset_entries = 0;
  let mut archive_writer_new = Writer::new().unwrap()
    .set_compression(ArchiveFilter::None) // could be imporoved later (libarchive-sys needs an upgrade ?)
    .set_format(ArchiveFormat::Zip);
  archive_writer_new
    .open_filename(&dataset_path.clone())
    .unwrap();
  // Bundle each usable status code:
  for status in vec![
    TaskStatus::NoProblem,
    TaskStatus::Warning,
    TaskStatus::Error,
  ]
  {
    let entries = backend.entries(&corpus, &service, &status);
    println!(
      "Entries found for severity {:?}: {:?}",
      status.to_key(),
      entries.len()
    );
    for entry in entries {
      // Let's open the zip file and grab the result from it
      if let Ok(archive_reader) =
        Reader::new()
          .unwrap()
          .support_filter_all()
          .support_format_all()
          .open_filename(&entry, 10240)
      {
        while let Ok(e) = archive_reader.next_header() {
          // Which file are we looking at?
          let pathname = e.pathname();
          let is_html = pathname.ends_with(".html");
          if !is_html {
            continue;
          }
          let mut raw_entry_data = Vec::new();
          while let Ok(chunk) = archive_reader.read_data(10240) {
            raw_entry_data.extend(chunk.into_iter());
          }
          let is_well_formed = match str::from_utf8(&raw_entry_data) {
            Ok(some_utf_string) => {
              if parser.is_well_formed_html(some_utf_string) {
                // well-formed, include in the dataset
                true
              } else {
                println!("-- Ill-formed XML: {:?}", entry);
                false // ill-formed, do nothing
              }
            }
            Err(_) => {
              println!("-- Ill-formed UTF8 archive data: {:?}", entry);
              false
            }
          };
          if is_well_formed {
            if let Some(cap) = entry_name_regex.captures(&entry) {
              let month_dir = cap.at(1).unwrap_or("monthXX");
              let paper_dir = cap.at(2).unwrap_or("paperXX");
              let dataset_path = status.to_key() + "/" + month_dir + "/" + paper_dir + ".html";
              println!("Writing: {:?} ", dataset_path);
              total_dataset_entries += 1;
              match archive_writer_new.write_header_new(
                &dataset_path,
                raw_entry_data.len() as i64,
              ) {
                Ok(_) => {}
                Err(e) => {
                  println!("Couldn't write header: {:?}", e);
                  break;
                }
              };
              match archive_writer_new.write_data(raw_entry_data) {
                Ok(_) => {}
                Err(e) => {
                  println!(
                    "Failed to write data to {:?} because {:?}",
                    dataset_path.clone(),
                    e
                  )
                }
              };
            }
          }
          break; // only one HTML file per archive
        }
      }
    }
  }
  let end_bundle = time::get_time();

  let bundle_duration = (end_bundle - start_bundle).num_milliseconds();
  println!(
    "-- Dataset bundler for corpus {:?} and service {:?} took {:?}ms",
    corpus.name,
    service.name,
    bundle_duration
  );
  println!("-- Bundled {:?} dataset entries.", total_dataset_entries);
}
