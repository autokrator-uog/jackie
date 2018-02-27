use std::vec::Vec;

use couchbase::{Bucket, N1qlResult};
use futures::{Stream};
use serde_json::{from_str};

use db::BUCKET_NAME;

#[derive(Clone, Debug, PartialEq, Deserialize, RustcEncodable)]
pub struct AggregationResult {
    pub event_type: String,
    pub event_count: i32
}

pub fn make_aggregations_of_event_types(bucket: &Bucket) -> Result<Vec<AggregationResult>, ()> {
    let query = format!("
        SELECT event_type, COUNT(event_type) event_count 
        FROM {}
        GROUP BY event_type
        ORDER BY event_count DESC
        ", BUCKET_NAME);
    
    let iter = bucket.query_n1ql(query).wait();
    let mut aggs: Vec<AggregationResult> = Vec::new();
    
    for row in iter {
        match row {
            Ok(N1qlResult::Meta(meta)) => {
                // we don't really care about this, just spit it out for debug
                debug!("raw meta received: meta='{:?}'", meta)
            },
            Ok(N1qlResult::Row(row)) => {
                debug!("raw row received: row='{}'", &row.as_ref());
                
                let result: AggregationResult = from_str(&row.as_ref()).unwrap();
                aggs.push(result);
            },
            Err(_) => return Err(())
        }
    }
    
    Ok(aggs)
}

#[derive(Clone, Deserialize, Serialize, RustcEncodable)]
pub struct Consistency {
    pub key: String,
    pub value: u32,
}

#[derive(Clone, Deserialize, Serialize, RustcEncodable)]
pub struct Event {
    pub consistency: Consistency,
    pub correlation_id: u32,
    // pub data: Value, // conflicts with...something, i cbb
    pub event_type: String,
    // We don't want this field going to Kafka.
    pub message_type: Option<String>,
    pub sender: String,
    pub session_id: Option<usize>,
    pub timestamp: String,
    pub timestamp_raw: Option<i64>,
}

#[derive(Clone, Deserialize, Serialize, RustcEncodable)]
pub struct EventResult {
    pub events: Event,
}

pub fn get_last_n_events(bucket: &Bucket, num_of_events: i32) -> Result<Vec<Event>, ()> {
    let query = format!("
        SELECT *
        FROM {0}
        ORDER BY timestamp_raw DESC
        LIMIT {1}
        ", BUCKET_NAME, num_of_events);

    let iter = bucket.query_n1ql(query).wait();
    let mut events: Vec<Event> = Vec::new();

    for row in iter {
        match row {
            Ok(N1qlResult::Meta(meta)) => {
                // we don't really care about this, just spit it out for debug
                debug!("raw meta received: meta='{:?}'", meta)
            },
            Ok(N1qlResult::Row(row)) => {
                debug!("raw row received: row='{}'", &row.as_ref());

                let result: EventResult = from_str(&row.as_ref()).unwrap();
                events.push(result.events);
            },
            Err(_) => return Err(())
        }
    }

    Ok(events)
}

pub fn get_events_for_consistency_key(bucket: &Bucket, consistency_key: &str) -> Result<Vec<Event>, ()> {
    let query = format!("
        SELECT *
        FROM {0}
        WHERE consistency.`KEY`i = \"{1}\"
        ORDER BY consistency.`VALUE`i
        ", BUCKET_NAME, consistency_key);
    
    let iter = bucket.query_n1ql(query).wait();
    let mut events: Vec<Event> = Vec::new();
    
    for row in iter {
        match row {
            Ok(N1qlResult::Meta(meta)) => {
                // we don't really care about this, just spit it out for debug
                debug!("raw meta received: meta='{:?}'", meta)
            },
            Ok(N1qlResult::Row(row)) => {
                debug!("raw row received: row='{}'", &row.as_ref());
                
                let result: EventResult = from_str(&row.as_ref()).unwrap();
                events.push(result.events);
            },
            Err(_) => return Err(())
        }
    }
    
    Ok(events)
}

pub fn get_events_for_correlation_id(bucket: &Bucket, correlation_id: &str) -> Result<Vec<Event>, ()> {
    let query = format!("
        SELECT *
        FROM {0}
        WHERE correlation_id = {1}
        ORDER BY timestamp_raw
        ", BUCKET_NAME, correlation_id);
    
    let iter = bucket.query_n1ql(query).wait();
    let mut events: Vec<Event> = Vec::new();
    
    for row in iter {
        match row {
            Ok(N1qlResult::Meta(meta)) => {
                // we don't really care about this, just spit it out for debug
                debug!("raw meta received: meta='{:?}'", meta)
            },
            Ok(N1qlResult::Row(row)) => {
                debug!("raw row received: row='{}'", &row.as_ref());
                
                let result: EventResult = from_str(&row.as_ref()).unwrap();
                events.push(result.events);
            },
            Err(_) => return Err(())
        }
    }
    
    Ok(events)
}
