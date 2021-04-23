#![feature(pattern)]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::nursery, clippy::pedantic)]
#![allow(clippy::module_inception, clippy::redundant_pub_crate)]
#![warn(missing_debug_implementations)]

pub mod adapters;
pub mod logic;

mod utils;
