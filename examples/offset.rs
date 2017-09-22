extern crate sntp_client;
extern crate time;

use sntp_client::*;
use time::*;

use std::net::*;

fn time_to_ntp(time: &Tm) -> NtpEpochTime {
    let t = time.to_timespec();

    NtpEpochTime::new(((NTP_TO_UNIX_EPOCH_SECONDS + t.sec as u64) * 1000) + (t.nsec as u64 / 1000000))
}

fn main() {
    let now = now_utc();
    println!("Local time: {:?}", now.to_timespec());

    let req = SntpData::new_request_sec(time_to_ntp(&now));
    println!("SNTP request: {:?}", req);
    
    let sntp_server = "0.pool.ntp.org:123";

    let send_addr = sntp_server;
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    println!("Communicating with SNTP server {}", send_addr);

    match socket.send_to(&req.get_data(), send_addr) {
        Ok(bytes) => { println!("Sent {} bytes", bytes); }
        Err(e) => { panic!("Failed to send an UDP reqeust: {}", e); }
    }
    
    let mut buf = [0; 48];
    socket.recv_from(&mut buf).unwrap();
    let received_at = now_utc();
    
    drop(socket);	

    let sntp_resp = SntpData::from_buffer(&buf).unwrap();
    println!("SNTP response: {:?}", sntp_resp);
    println!("Local system time offset: {:?} ms", sntp_resp.local_time_offset(time_to_ntp(&received_at)));
}
