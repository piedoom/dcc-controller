#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(associated_type_defaults)]
#![feature(async_closure)]

use dcc_rs::{
    bitvec::{BitArr, order::Msb0},
    packets::MAX_BITS,
};
pub mod pins;
pub mod tasks;

#[derive(Default)]
pub struct Buffer(pub BitArr!(for MAX_BITS * 4, in u8, Msb0)); // Arbitrary values
