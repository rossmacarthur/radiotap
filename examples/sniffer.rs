//! This example opens a packet capture on the given interface, and prints out
//! the radiotap capture for the first 100 captured packets.
//!
//! On some macOS systems you might need to first put the Wi-Fi interface into
//! monitor mode using: `airport sniff <channel num>`

use std::env;

use anyhow::Context;

const DLT_IEEE802_11_RADIO: i32 = 127;

fn main() -> anyhow::Result<()> {
    // Use first argument interface if passed in, else default to "en0" or "wlan0"
    let arg = env::args().nth(1);
    let device = arg.as_deref().unwrap_or_else(|| {
        if cfg!(target_os = "macos") {
            "en0"
        } else {
            "wlan0"
        }
    });

    // Open packet capture and set data link to 802.11 radiotap
    let mut cap = pcap::Capture::from_device(device)?
        .timeout(1)
        .rfmon(cfg!(target_os = "macos"))
        .open()?;
    cap.set_datalink(pcap::Linktype(DLT_IEEE802_11_RADIO))?;

    let mut count = 0;
    while count < 100 {
        match cap.next() {
            Ok(packet) => {
                // Actually parse the radiotap header of the packet!
                let header = radiotap::parse(&packet).context("failed to parse radiotap header")?;
                println!("{:#?}\n", header);
                count += 1;
            }
            Err(pcap::Error::TimeoutExpired) => continue,
            Err(err) => return Err(err).context("unexpected pcap error"),
        }
    }
    Ok(())
}
