//! UUIDv1 Batteries
//!
//! This library provides convenient methods for generating version 1 UUIDs
//! and ordering them.

use std::fmt;
use std::fs::File;
use std::io::Read;

use chrono::Utc;
use uuid::v1::{Context, Timestamp};
use uuid::Uuid;

/// https://www.percona.com/blog/2014/12/19/store-uuid-optimized-way/
#[derive(Debug)]
pub struct OrderedUuid(Uuid);

struct Config {
    ctx: Context,
    node: Uuid,
}

lazy_static::lazy_static! {
    static ref CONFIG: Config = {
        let node = {
            let mut machine_id = None;
            if let Ok(mut file) = File::open("/etc/machine-id") {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    if let Ok(id) = Uuid::parse_str(contents.trim()) {
                        machine_id = Some(id);
                    }
                }
            }
            match machine_id {
                Some(id) => id,
                None => Uuid::new_v4(),
            }
        };
        let ctx = Context::new(rand::random());
        Config { ctx, node }
    };
}

pub trait Uuid1 {
    fn v1() -> Uuid;
    fn ordered(&self) -> Option<OrderedUuid>;
}

impl Uuid1 for Uuid {
    fn v1() -> Uuid {
        let ctx = &CONFIG.ctx;
        let node_id = &CONFIG.node.as_bytes()[..6];
        let now = Utc::now();
        let seconds = now.timestamp() as u64;
        let nsecs = now.timestamp_subsec_nanos();
        let ts = Timestamp::from_unix(ctx, seconds, nsecs);
        Uuid::new_v1(ts, node_id).unwrap()
    }

    fn ordered(&self) -> Option<OrderedUuid> {
        if self.get_version_num() == 1 {
            let ordered: [u8; 16] = {
                let bytes = self.as_bytes();
                unsafe {
                    [
                        *bytes.get_unchecked(6),
                        *bytes.get_unchecked(7),
                        *bytes.get_unchecked(4),
                        *bytes.get_unchecked(5),
                        *bytes.get_unchecked(0),
                        *bytes.get_unchecked(1),
                        *bytes.get_unchecked(2),
                        *bytes.get_unchecked(3),
                        *bytes.get_unchecked(8),
                        *bytes.get_unchecked(9),
                        *bytes.get_unchecked(10),
                        *bytes.get_unchecked(11),
                        *bytes.get_unchecked(12),
                        *bytes.get_unchecked(13),
                        *bytes.get_unchecked(14),
                        *bytes.get_unchecked(15),
                    ]
                }
            };
            let uuid = Uuid::from_bytes(ordered);
            return Some(OrderedUuid(uuid));
        }
        None
    }
}

impl fmt::Display for OrderedUuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::Uuid1;
    use uuid::Uuid;

    #[test]
    fn ordered_uuid_is_some() {
        let uuid = Uuid::v1();
        assert!(uuid.ordered().is_some());
    }
}
