use anchor_lang::prelude::*;

#[constant]
pub const SEED: &[u8] = b"favorites";
pub const SEED_V1: &[u8] = b"favoritesV1";
pub const ANCHOR_DISCRIMINATOR_SIZE:usize = 8;