use ::std::str::FromStr;
use ::std::fmt::{Display, Formatter, Error as FmtError};
use ::error::KristforgeError;
use ::indicatif::ProgressBar;

/// A krist address
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Address(pub [u8; Address::LENGTH]);

impl Address {
	pub const LENGTH: usize = 10;
}

impl FromStr for Address {
	type Err = KristforgeError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.len() != Address::LENGTH {
			Err(KristforgeError::InvalidAddressLength(s.len()))
		} else {
			let mut data = [0u8; Address::LENGTH];
			data.copy_from_slice(s.as_bytes());
			Ok(Address(data))
		}
	}
}

/// The current mining target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
	pub work: i64,
	pub prev_block: [u8; Target::BLOCK_LEN]
}

impl Target {
	pub const BLOCK_LEN: usize = 12;
}

/// A solution for a specific target
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solution {
	pub target: Target,
	pub address: Address,
	pub nonce: [u8; Solution::NONCE_LEN]
}

impl Solution {
		pub const NONCE_LEN: usize = 15;
}
