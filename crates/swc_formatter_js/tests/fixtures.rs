#[path = "support/prettier_options.rs"]
mod prettier_options;

#[path = "fixtures/mod.rs"]
mod fixtures;

include!(concat!(env!("OUT_DIR"), "/formatter_fixtures.rs"));
