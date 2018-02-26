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
