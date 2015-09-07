#![no_std]

#![feature(core, alloc, no_std, macro_reexport, unboxed_closures, collections, convert, hash, step_by)]

#[macro_use(write)]
extern crate core;
extern crate alloc;
extern crate collections;


use core::prelude::*;
use core::hash::Hasher;
use core::hash::SipHasher;
use core::array::FixedSizeArray;
use core::fmt::{Formatter};

use collections::vec::*;

pub const NtpToUnixEpochSeconds: u64 = 0x83AA7E80;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NtpEpochTime(u64);

impl NtpEpochTime {
	pub fn new(ms: u64) -> NtpEpochTime {
		NtpEpochTime(ms)
	}

	pub fn from_unix_seconds(unix: u64) -> NtpEpochTime {
		NtpEpochTime((NtpToUnixEpochSeconds + unix) * 1000)
	}

	pub fn to_u64(&self) -> u64 {
		let NtpEpochTime(ms) = *self;
		ms
	}

	pub fn to_unix_seconds(&self) -> u64 {
		(self.to_u64() / 1000) - NtpToUnixEpochSeconds
	}
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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

	pub fn new_request_sec(now: NtpEpochTime) -> SntpData {
		let mut data = SntpData::new();
		data.set_mode(SntpMode::Client);
		data.set_version(4);
		data.set_transmit_time(now);
		data
	}

	pub fn from_buffer(buff: &[u8]) -> Option<SntpData> {
		if buff.len() != 48 { return None; }
		let mut s = SntpData::new();
		s.data.clone_from_slice(buff);
		Some(s)
	}


	pub fn get_version(&self) -> u8 {
		(self.data[0] & 0x38) >> 3
	}

	pub fn set_version(&mut self, version: u8) {
		self.data[0] = ((self.data[0]) & !0x38) | ((version & 0x7) << 3);
	}

	pub fn get_reference_timestamp(&self) -> NtpEpochTime {
		SntpData::data_to_ms(&self.data[16..24])
	}

	// T1
	pub fn get_originate_timestamp(&self) -> NtpEpochTime {
		SntpData::data_to_ms(&self.data[24..32])
	}

	// T2
	pub fn get_receive_time(&self) -> NtpEpochTime {
		SntpData::data_to_ms(&self.data[32..40])
	}

	// T3
	pub fn get_transmit_time(&self) -> NtpEpochTime {
		SntpData::data_to_ms(&self.data[40..48])
	}	


	// T3
	pub fn set_transmit_time(&mut self, ms: NtpEpochTime) {
		let d = SntpData::ms_to_data(ms);
		let ref mut t = &mut self.data[40..48];

		for i in 0..d.len() {
			t[i] = d[i];
		}
	}

	

	pub fn set_mode(&mut self, mode: SntpMode) {
		self.data[0] = ((self.data[0]) & !0x7) | (0x7 & mode.to_val());
	}

	pub fn get_mode(&self) -> SntpMode {
		SntpMode::from_val(self.data[0] & 0x7)
	}

	pub fn data_to_ms(data: &[u8]) -> NtpEpochTime {
		if data.len() != 8 { return NtpEpochTime::new(0); }

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

		NtpEpochTime::new(int_part * 1000 + (frac_part * 1000) / 0x100000000)
	}

	pub fn ms_to_data(ms: NtpEpochTime) -> [u8; 8] {
		let int_part = ms.to_u64() / 1000;
		let frac_part = ((ms.to_u64() % 1000) * 0x100000000) / 1000;

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

	pub fn local_time_offset(&self, response_received_at: NtpEpochTime) -> i64 {
		let s = (self.get_receive_time().to_u64() as i64 - self.get_originate_timestamp().to_u64() as i64) +
			    (self.get_transmit_time().to_u64() as i64 - response_received_at.to_u64() as i64);
		s / 2
	}

	pub fn get_data(&self) -> &[u8] {
		&self.data
	}
}

impl core::fmt::Debug for SntpData {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(f, "reference time: {:?}, t1: {:?}, t2: {:?}, t3: {:?}", 
        	self.get_reference_timestamp(),
        	self.get_originate_timestamp(),
        	self.get_receive_time(),
        	self.get_transmit_time()
        	)
    }
}





// for tests
#[cfg(test)]
#[macro_use(println, assert_eq, print, panic, try, panic)]
extern crate std;



#[cfg(test)]
extern crate time;

#[cfg(test)]
mod tests {
	

	use super::*;
	use core::prelude::*;
	use std::prelude::*;
	use collections::vec::Vec;
	use time::*;
	

	#[test]
	fn test_ms_conv() {
		let ms = NtpEpochTime::new(5229834209);

		let data = SntpData::ms_to_data(ms);

		let ms_conv = SntpData::data_to_ms(&data);

		assert_eq!(ms, ms_conv);
	}

	#[test]
	fn sntp_data() {
		let mut data = SntpData::new();
		data.set_mode(SntpMode::Client);
		data.set_version(4);
		data.set_transmit_time(NtpEpochTime::new(1001));

		assert_eq!(NtpEpochTime::new(1001), data.get_transmit_time());
		assert_eq!(SntpMode::Client, data.get_mode());
		assert_eq!(4, data.get_version());

	}

	fn time_to_ntp(time: &Tm) -> NtpEpochTime {
		let t = time.to_timespec();

		NtpEpochTime::new(((NtpToUnixEpochSeconds + t.sec as u64) * 1000) + (t.nsec as u64 / 1000000))
	}

	use std::net::*;

	#[test]
	fn sntp_udp() {
		
		let now = now_utc();
		println!("{:?}", now.to_timespec());

		let req = SntpData::new_request_sec(time_to_ntp(&now));
		println!("sntp request: {:?}", req);
		
		let send_addr = "0.pool.ntp.org:123";
		let mut socket = UdpSocket::bind("0.0.0.0:0").unwrap();

		match socket.send_to(&req.data[..], send_addr) {
			Ok(bytes) => { println!("Sent {} bytes", bytes); }
			_ => { panic!("fail"); }
		}
		
		let mut buf = [0; 48];
		let (amt, src) = socket.recv_from(&mut buf).unwrap();

		let received_at = now_utc();

		{
			let mut v = Vec::new();
			v.push_all(&buf);
			println!("net received: {:?}", v);
		}

		drop(socket);

	

		let sntp_resp = SntpData::from_buffer(&buf).unwrap();
		println!("sntp response: {:?}", sntp_resp);
		println!("local system time offset: {:?} ms", sntp_resp.local_time_offset(time_to_ntp(&received_at)));


	}
}

