#[macro_use] extern crate nickel;
#[macro_use] extern crate clap;
#[macro_use] extern crate log;
extern crate fern;
extern crate chrono;
extern crate colored;
extern crate couchbase;
extern crate futures;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate rustc_serialize;

mod logging;
mod db;
mod queries;

use std::collections::HashMap;

use clap::{Arg, App};
use log::LogLevelFilter;
use nickel::{Nickel, HttpRouter, StaticFilesHandler};

use db::{BUCKET_NAME, connect_to_bucket};
use logging::configure_logging;

#[allow(resolve_trait_on_defaulted_unit)]
#[allow(unreachable_code)]
fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("log-level")
             .short("l")
             .long("log-level")
             .help("Log level")
             .default_value("info")
             .possible_values(&["off", "trace", "debug", "info", "warn", "error"])
             .takes_value(true))
        .arg(Arg::with_name("port")
             .short("p")
             .long("port")
             .help("Port to bind server to")
             .default_value("6767")
             .takes_value(true))
        .arg(Arg::with_name("couchbase_host")
            .short("cb")
            .long("couchbase-host")
            .help("The hostname for the couchbase DB.")
            .default_value("couchbase.db")
            .takes_value(true))
        .get_matches();
    
    let level = value_t!(matches, "log-level", LogLevelFilter).unwrap_or(LogLevelFilter::Trace);
    configure_logging(level);
    
    // init couchbase connection to bucket
    let couchbase_host = matches.value_of("couchbase_host").expect("Error in clap.").to_string();
    let cbh1 = couchbase_host.clone();
    let cbh2 = couchbase_host.clone();
    let cbh3 = couchbase_host.clone();
    let cbh4 = couchbase_host.clone();
    // I know this is ugly... I don't care, I'm beyond caring with Rust now.
    
    
    // init HTTP server
    let mut server = Nickel::new();
    server.utilize(StaticFilesHandler::new("src/static/"));

    // index page
    server.get("/", middleware! { |_, res|
        let bucket = match connect_to_bucket(&cbh1, BUCKET_NAME) {
            Ok(b) => b,
            Err(_) => {
                panic!("Couldn't connect to couchbase bucket... exiting now.")
            }
        };
        
        let mut data: HashMap<&str, Vec<queries::Event>> = HashMap::new();
        
        let events = queries::get_last_n_events(&bucket, 20).expect("Last 20 events failed!");
        data.insert("events", events);
        
        trace!("Rendering index.html...");
        return res.render("src/templates/index.html", &data)
    });
    
    server.get("/totals", middleware! { |_, res|
        let bucket = match connect_to_bucket(&cbh2, BUCKET_NAME) {
            Ok(b) => b,
            Err(_) => {
                panic!("Couldn't connect to couchbase bucket... exiting now.")
            }
        };
        
        let mut data: HashMap<&str, Vec<queries::AggregationResult>> = HashMap::new();
        
        let aggs = queries::make_aggregations_of_event_types(&bucket).expect("Aggregations failed!");
        data.insert("aggs", aggs);
        
        trace!("Rendering index.html...");
        return res.render("src/templates/totals.html", &data)
    });
    
    
    server.get("/consistency_key/:ckey/events", middleware! { |req, res|
        let consistency_key = req.param("ckey").unwrap();
        
        let bucket = match connect_to_bucket(&cbh3, BUCKET_NAME) {
            Ok(b) => b,
            Err(_) => {
                panic!("Couldn't connect to couchbase bucket... exiting now.")
            }
        };
        
        let mut data: HashMap<&str, Vec<queries::Event>> = HashMap::new();
        
        let events = queries::get_events_for_consistency_key(&bucket, consistency_key).expect("Failed to get events for consistency_key");
        data.insert("events", events);
        
        return res.render("src/templates/consistency_key_view.html", &data)
    });
    
    server.get("/correlation_id/:cid/events", middleware! { |req, res| 
        let correlation_id = req.param("cid").unwrap();
        
        let bucket = match connect_to_bucket(&cbh4, BUCKET_NAME) {
            Ok(b) => b,
            Err(_) => {
                panic!("Couldn't connect to couchbase bucket... exiting now.")
            }
        };
        
        let mut data: HashMap<&str, Vec<queries::Event>> = HashMap::new();
        
        let events = queries::get_events_for_correlation_id(&bucket, correlation_id).expect("Failed to get events for consistency_key");
        data.insert("events", events);
        
        return res.render("src/templates/correlation_id_view.html", &data)
    });
    
    server.listen(
        format!("0.0.0.0:{}", matches.value_of("port").expect("No port specified and error in clap config!")))
        .expect("Unable to start server!");
}
