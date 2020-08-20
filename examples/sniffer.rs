//! This example opens a packet capture on the given interface, and prints out
//! the radiotap capture for the first 100 captured packets.
//!
//! On some macOS systems you might need to first put the Wi-Fi interface into

use std::env;

const DLT_IEEE802_11_RADIO: i32 = 127;

fn main() {
    // Use first argument interface if passed in, else default to "en0" or "wlan0"
    let device = if let Some(arg) = env::args().nth(1) {
        arg
    } else {
        if cfg!(target_os = "macos") {
            "en0"
        } else {
            "wlan0"
        }
        .to_string()
    };

    // Open packet capture and set data link to 802.11 radiotap
    let mut cap = pcap::Capture::from_device(&device[..])
        .unwrap()
        .timeout(1)
        .rfmon(cfg!(target_os = "macos"))
        .open()
        .unwrap();
    cap.set_datalink(pcap::Linktype(DLT_IEEE802_11_RADIO))
        .unwrap();

    let mut count = 0;
    // Print out the first 100 radiotap headers of packets
    while count < 100 {
        // Get a packet from the interface
        match cap.next() {
            Ok(packet) => {
                // Parse the radiotap header of the packet!
                if let Ok(header) = radiotap::parse(&packet) {
                    println!("{:?}\n", header);
                    count += 1;
                }
            }

            Err(pcap::Error::TimeoutExpired) => continue,

            Err(e) => {
                println!("Unexpected error: {:?}", e);
                break;
            }
        }
    }
}
