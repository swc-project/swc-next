#[path = "fixtures/mod.rs"]
mod fixtures;

include!(concat!(env!("OUT_DIR"), "/formatter_fixtures.rs"));
