#![no_std]

#![feature(core, alloc, no_std, macro_reexport, unboxed_closures, collections, convert, hash, step_by)]

extern crate core;
extern crate alloc;
extern crate collections;

use core::prelude::*;
use core::hash::Hasher;
use core::hash::SipHasher;
use core::array::FixedSizeArray;

use collections::vec::*;


pub enum SntpMode {
	Unknown,
	SymmetricActive,
	SymmetricPassive,
	Client,
	Server,
	Broadcast
}

impl SntpMode {
	pub fn to_val(&self) -> u8 {
		match *self {
			SntpMode::Unknown => 0,
			SntpMode::SymmetricActive => 1,
			SntpMode::SymmetricPassive => 2,
			SntpMode::Client => 3,
			SntpMode::Server => 4,
			SntpMode::Broadcast => 5
		}
	}

	pub fn from_val(v: u8) -> SntpMode {
		match v {
			1 => SntpMode::SymmetricActive,
			2 => SntpMode::SymmetricPassive,
			3 => SntpMode::Client,
			4 => SntpMode::Server,
			5 => SntpMode::Broadcast,
			_ => SntpMode::Unknown
		}
	}
}

pub struct SntpData {
	data: [u8; 48]
}

impl SntpData {
	pub fn new() -> SntpData {
		SntpData {
			data: [0; 48]
		}
	}

	pub fn set_mode(&mut self, mode: SntpMode) {
		self.data[0] = self.data[0] & mode.to_val();
	}

	pub fn get_mode(&self) -> SntpMode {
		SntpMode::from_val(self.data[0] & 0x7)
	}

	pub fn data_to_ms(data: &[u8; 8]) -> u64 {
		let int_part = {
			let mut p: u64 = 0;
			for i in 0..4 {
				p = (256 * p) + data[i] as u64;
			}
			p
		};

		let frac_part = {
			let mut f: u64 = 0;
			for i in 4..8 {
				f = (256 * f) + data[i] as u64;
			}
			f + 1
		};

		int_part * 1000 + (frac_part * 1000) / 0x100000000
	}

	pub fn ms_to_data(ms: u64) -> [u8; 8] {
		let int_part = ms / 1000;
		let frac_part = ((ms % 1000) * 0x100000000) / 1000;

		let mut data = [0; 8];

		let mut temp = int_part;
		for i in (3..-1).step_by(-1 as i16) {
			data[i as usize] = (temp % 256) as u8;			
			temp = temp / 256;
		}

		let mut temp = frac_part;
		for i in (7..3).step_by(-1 as i16) {
			data[i as usize] = (temp % 256) as u8;
			temp = temp / 256;
		}

		data
	}
}





// for tests
#[cfg(test)]
#[macro_use(println, assert_eq, print, panic)]
extern crate std;

#[cfg(test)]
mod tests {
	use super::*;
	use core::prelude::*;
	use std::prelude::*;
	use collections::vec::Vec;

	#[test]
	fn test_ms_conv() {
		let ms: u64 = 5229834209;

		let data = SntpData::ms_to_data(ms);

		//println!("data: {:?}", data);

		let ms_conv = SntpData::data_to_ms(&data);

		assert_eq!(ms, ms_conv);
	}

	#[test]
	fn sntp1() {
		let mut data = SntpData::new();
		data.set_mode(SntpMode::Client);

		//println!("data: {:?}", data);
	}
}

