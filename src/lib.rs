#![feature(pattern)]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::nursery, clippy::pedantic)]
#![allow(
    clippy::module_inception,
    clippy::redundant_pub_crate,
    clippy::module_name_repetitions
)]
#![warn(missing_debug_implementations)]

pub mod adapters;
pub mod logic;

pub mod utils;
