use std::{thread, time};

use couchbase::{Bucket, Cluster, CouchbaseError};

pub const BUCKET_NAME: &str = "events";
const MAX_RETRIES: u8 = 60;
const RETRY_INTERVAL_MILLIS: u64 = 1000;

pub fn connect_to_bucket(couchbase_host: &String, bucket_name: &str) -> Result<Bucket, ()> {
    // This is simply a state object, it doesn't actually initiate connections.
    let mut cluster = Cluster::new(&couchbase_host[..]).expect("Cannot create couchbase cluster object!");
    cluster.authenticate("connect", "connect");
    let cluster = cluster;

    // Retry logic on opening bucket.
    let mut retries = MAX_RETRIES;
    let mut bucket = cluster.open_bucket(bucket_name, None);

    while bucket.is_err() && retries > 0 {
        match bucket {
            Ok(_) => {
                panic!("Bucket should be err... this shouldn't happen!");
            },
            // This is the error that is called if the bucket does not exist, somehow...
            Err(CouchbaseError::AuthFailed) => {
                warn!("the bucket does not exist. waiting for it to be created: \
                      bucket='{}' retries_remaining='{}'", bucket_name, retries);
            },
            Err(err) => {
                error!("failed to connect to couchbase: bucket='{}' host='{}' \
                       error='{}' retries_remaining='{}'",
                       bucket_name, couchbase_host, err, retries);
            },
        }

        retries -= 1;
        bucket = cluster.open_bucket(bucket_name, None);
        thread::sleep(time::Duration::from_millis(RETRY_INTERVAL_MILLIS));
    }

    let bucket = match bucket {
        Ok(bucket) => {
            info!("successfully connected to couchbase bucket: bucket='{}'", bucket_name);
            bucket
        },
        Err(e) => {
            error!("Error even after retries... error='{}'", e);
            return Err(());
        }
    };

    Ok(bucket)
}