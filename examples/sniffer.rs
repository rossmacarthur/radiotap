//! This example opens a packet capture on the given interface, and prints out the Radiotap capture
//! for the first 100 captured packets.

extern crate pcap;
extern crate radiotap;
use std::env;

fn main() {
    // Use first argument interface if passed in, else default to "en0"
    let device = if let Some(arg) = env::args().nth(1) {
        arg
    } else {
        "en0".to_owned()
    };

    // Open packet capture and set data link to 802.11 Radiotap
    let mut cap = pcap::Capture::from_device(&device[..])
        .unwrap()
        .timeout(1)
        .rfmon(true)
        .open()
        .unwrap();
    cap.set_datalink(pcap::Linktype(127)).unwrap(); // DLT_IEEE802_11_RADIO = 127

    let mut count = 0;
    // Print out the first 100 Radiotap headers of packets
    while count < 100 {
        // Get a packet from the interface
        match cap.next() {
            // We captured a packet
            Ok(packet) => {
                // Parse the radiotap header of the packet
                let radiotap_header = radiotap::Radiotap::from_bytes(&packet);
                // If it parsed correctly, then print out the radiotap header
                if radiotap_header.is_ok() {
                    println!("{:?}\n", radiotap_header);
                    count += 1;
                }
            }
            // There were no packets on the interface before the timeout
            Err(pcap::Error::TimeoutExpired) => continue,
            // There was an unknown error
            Err(e) => {
                println!("Unexpected error: {:?}", e);
                break;
            }
        }
    }
}
