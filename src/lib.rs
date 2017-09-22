#![no_std]

use core::fmt::Formatter;

pub const NTP_TO_UNIX_EPOCH_SECONDS: u64 = 0x83AA7E80;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NtpEpochTime(u64);

impl NtpEpochTime {
	pub fn new(ms: u64) -> NtpEpochTime {
		NtpEpochTime(ms)
	}

	pub fn from_unix_seconds(unix: u64) -> NtpEpochTime {
		NtpEpochTime((NTP_TO_UNIX_EPOCH_SECONDS + unix) * 1000)
	}

	pub fn to_u64(&self) -> u64 {
		let NtpEpochTime(ms) = *self;
		ms
	}

	pub fn to_unix_seconds(&self) -> u64 {
		(self.to_u64() / 1000) - NTP_TO_UNIX_EPOCH_SECONDS
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

	pub fn new_request_sec(now: NtpEpochTime) -> Self {
		let mut data = SntpData::new();
		data.set_mode(SntpMode::Client);
		data.set_version(4);
		data.set_transmit_time(now);
		data
	}

	pub fn from_buffer(buff: &[u8]) -> Result<SntpData, ()> {
		if buff.len() != 48 { return Err(()); }
		let mut s = SntpData::new();
		s.data.clone_from_slice(buff);
		Ok(s)
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
		for i in &[3, 2, 1, 0] {
			data[*i] = (temp % 256) as u8;			
			temp = temp / 256;
		}

		let mut temp = frac_part;
		for i in &[7, 6, 5, 4] {
			data[*i] = (temp % 256) as u8;
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

#[test]
fn test_ms_conv() {
	let ms = NtpEpochTime::new(5229834209);

	let data = SntpData::ms_to_data(ms);
	assert_eq!(&[0, 79, 205, 10, 53, 129, 6, 36], &data);
	
	let ms_conv = SntpData::data_to_ms(&data);
	assert_eq!(ms, ms_conv);
}

#[test]
fn sntp_data_gen() {
	let mut data = SntpData::new();
	data.set_mode(SntpMode::Client);
	data.set_version(4);
	data.set_transmit_time(NtpEpochTime::new(1001));

	assert_eq!(NtpEpochTime::new(1001), data.get_transmit_time());
	assert_eq!(SntpMode::Client, data.get_mode());
	assert_eq!(4, data.get_version());
}

#[test]
fn sntp_data_parse() {
	let buf = [36, 2, 3, 232, 0, 0, 1, 39, 0, 0, 9, 20, 162, 23, 41, 56, 221, 111, 129, 38, 220, 243, 246, 238, 221, 111, 132, 223, 232, 180, 57, 88, 221, 111, 132, 223, 223, 210, 132, 17, 221, 111, 132, 223, 223, 213, 89, 109];

	let sntp_resp = SntpData::from_buffer(&buf).unwrap();

	assert_eq!(NtpEpochTime(3715072294863), sntp_resp.get_reference_timestamp());
	assert_eq!(NtpEpochTime(3715073247909), sntp_resp.get_originate_timestamp());
	assert_eq!(NtpEpochTime(3715073247874), sntp_resp.get_receive_time());
	assert_eq!(NtpEpochTime(3715073247874), sntp_resp.get_transmit_time());
}

