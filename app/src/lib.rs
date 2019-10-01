#![feature(async_await)]
#![feature(test)]
#![feature(rustc_private)]

extern crate libc;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate futures;
extern crate async_fs;
extern crate opencv;
extern crate actix_web;
extern crate actix;
extern crate actix_rt;
extern crate num_cpus;
extern crate env_logger;
extern crate actix_files;
extern crate serde;
extern crate crypto;
extern crate serde_derive;
extern crate serde_json;
extern crate base64;
extern crate actix_multipart;
extern crate mime;
extern crate futures_cpupool;
extern crate bytes;
extern crate graceful;
extern crate config;
extern crate positioned_io;

#[macro_use] pub mod utils;
pub mod routes;
pub mod resources;
