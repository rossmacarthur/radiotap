//! Radiotap field definitions and parsers.

pub mod ext;

use bitops::BitOps;
use byteorder::{ReadBytesExt, LE};
use std::io::{Cursor, Read};

use crate::{field::ext::*, Error, Result};

type OUI = [u8; 3];

/// The type of Radiotap field.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Kind {
    TSFT,
    Flags,
    Rate,
    Channel,
    FHSS,
    AntennaSignal,
    AntennaNoise,
    LockQuality,
    TxAttenuation,
    TxAttenuationDb,
    TxPower,
    Antenna,
    AntennaSignalDb,
    AntennaNoiseDb,
    RxFlags,
    TxFlags,
    RTSRetries,
    DataRetries,
    XChannel,
    MCS,
    AMPDUStatus,
    VHT,
    Timestamp,
    VendorNamespace(Option<VendorNamespace>),
}

impl Kind {
    pub fn new(value: u8) -> Result<Kind> {
        Ok(match value {
            0 => Kind::TSFT,
            1 => Kind::Flags,
            2 => Kind::Rate,
            3 => Kind::Channel,
            4 => Kind::FHSS,
            5 => Kind::AntennaSignal,
            6 => Kind::AntennaNoise,
            7 => Kind::LockQuality,
            8 => Kind::TxAttenuation,
            9 => Kind::TxAttenuationDb,
            10 => Kind::TxPower,
            11 => Kind::Antenna,
            12 => Kind::AntennaSignalDb,
            13 => Kind::AntennaNoiseDb,
            14 => Kind::RxFlags,
            15 => Kind::TxFlags,
            16 => Kind::RTSRetries,
            17 => Kind::DataRetries,
            18 => Kind::XChannel,
            19 => Kind::MCS,
            20 => Kind::AMPDUStatus,
            21 => Kind::VHT,
            22 => Kind::Timestamp,
            _ => {
                return Err(Error::UnsupportedField);
            }
        })
    }

    /// Returns the align value for the field.
    pub fn align(&self) -> u64 {
        match *self {
            Kind::TSFT | Kind::Timestamp => 8,
            Kind::XChannel | Kind::AMPDUStatus => 4,
            Kind::Channel
            | Kind::FHSS
            | Kind::LockQuality
            | Kind::TxAttenuation
            | Kind::TxAttenuationDb
            | Kind::RxFlags
            | Kind::TxFlags
            | Kind::VHT
            | Kind::VendorNamespace(_) => 2,
            _ => 1,
        }
    }

    /// Returns the size of the field.
    pub fn size(&self) -> usize {
        match *self {
            Kind::VHT | Kind::Timestamp => 12,
            Kind::TSFT | Kind::AMPDUStatus | Kind::XChannel => 8,
            Kind::VendorNamespace(_) => 6,
            Kind::Channel => 4,
            Kind::MCS => 3,
            Kind::FHSS
            | Kind::LockQuality
            | Kind::TxAttenuation
            | Kind::TxAttenuationDb
            | Kind::RxFlags
            | Kind::TxFlags => 2,
            _ => 1,
        }
    }
}

pub trait Field {
    fn from_bytes(input: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

/// Parse any `Field` and return a `Result<T>`.
pub fn from_bytes<T>(input: &[u8]) -> Result<T>
where
    T: Field,
{
    T::from_bytes(input)
}

/// Parse any `Field` and return a `Result<Some<T>>`.
pub fn from_bytes_some<T>(input: &[u8]) -> Result<Option<T>>
where
    T: Field,
{
    Ok(Some(T::from_bytes(input)?))
}

/// The Radiotap header, contained in all Radiotap captures.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Header {
    /// The Radiotap version, only version 0 is supported.
    pub version: u8,
    /// The length of the entire Radiotap capture.
    pub length: usize,
    /// The size of the Radiotap header.
    pub size: usize,
    /// The fields present in the Radiotap capture.
    pub present: Vec<Kind>,
}

impl Field for Header {
    fn from_bytes(input: &[u8]) -> Result<Header> {
        let mut cursor = Cursor::new(input);

        let version = cursor.read_u8()?;
        if version != 0 {
            // We only support version 0
            return Err(Error::UnsupportedVersion);
        }

        cursor.read_u8()?; // Account for 1 byte padding field

        let length = cursor.read_u16::<LE>()?;
        if input.len() < length as usize {
            return Err(Error::InvalidLength);
        }

        let mut present;
        let mut present_count = 0;
        let mut vendor_namespace = false;
        let mut kinds = Vec::new();

        loop {
            present = cursor.read_u32::<LE>()?;

            if !vendor_namespace {
                for bit in 0..29 {
                    if present.is_bit_set(bit) {
                        match Kind::new(present_count * 32 + bit) {
                            Ok(kind) => {
                                kinds.push(kind);
                            }
                            Err(Error::UnsupportedField) => {
                                // Does not matter, we will just parse the ones we can
                            }
                            Err(e) => return Err(e),
                        }
                    }
                }
            }

            // Need to move to radiotap namespace
            if present.is_bit_set(29) {
                present_count = 0;
                vendor_namespace = false;

            // Need to move to vendor namespace
            } else if present.is_bit_set(30) {
                present_count = 0;
                vendor_namespace = true;
                // We'll figure out what namespace it is later, just use none
                kinds.push(Kind::VendorNamespace(None))

            // Need to stay in the same namespace
            } else {
                present_count += 1;
            }

            // More present words do not exist
            if !present.is_bit_set(31) {
                break;
            }
        }

        Ok(Header {
            version,
            length: length as usize,
            size: cursor.position() as usize,
            present: kinds,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VendorNamespace {
    pub oui: OUI,
    pub sub_namespace: u8,
    pub skip_length: u16,
}

impl Field for VendorNamespace {
    fn from_bytes(input: &[u8]) -> Result<VendorNamespace> {
        let mut cursor = Cursor::new(input);
        let mut oui = [0; 3];
        cursor.read(&mut oui)?;
        let sub_namespace = cursor.read_u8()?;
        let skip_length = cursor.read_u16::<LE>()?;
        Ok(VendorNamespace {
            oui,
            sub_namespace,
            skip_length,
        })
    }
}

/// Value in microseconds of the MAC’s 64-bit 802.11 Time Synchronization
/// Function timer when the first bit of the MPDU arrived at the MAC. For
/// received frames only.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TSFT {
    pub value: u64,
}

impl Field for TSFT {
    fn from_bytes(input: &[u8]) -> Result<TSFT> {
        let value = Cursor::new(input).read_u64::<LE>()?;
        Ok(TSFT { value })
    }
}

/// Properties of transmitted and received frames.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Flags {
    /// The frame was sent/received during CFP.
    pub cfp: bool,
    /// The frame was sent/received with short preamble.
    pub preamble: bool,
    /// The frame was sent/received with WEP encryption.
    pub wep: bool,
    /// The frame was sent/received with fragmentation.
    pub fragmentation: bool,
    /// The frame includes FCS.
    pub fcs: bool,
    /// The frame has padding between 802.11 header and payload (to 32-bit
    /// boundary).
    pub data_pad: bool,
    /// The frame failed FCS check.
    pub bad_fcs: bool,
    /// The frame used short guard interval (HT).
    pub sgi: bool,
}

impl Field for Flags {
    fn from_bytes(input: &[u8]) -> Result<Flags> {
        let flags = Cursor::new(input).read_u8()?;
        Ok(Flags {
            cfp: flags.is_flag_set(0x01),
            preamble: flags.is_flag_set(0x02),
            wep: flags.is_flag_set(0x04),
            fragmentation: flags.is_flag_set(0x08),
            fcs: flags.is_flag_set(0x10),
            data_pad: flags.is_flag_set(0x20),
            bad_fcs: flags.is_flag_set(0x40),
            sgi: flags.is_flag_set(0x80),
        })
    }
}

/// The legacy data rate in Mbps. Usually only one of the
/// [Rate](struct.Rate.html), [MCS](struct.MCS.html), and [VHT](struct.VHT.html)
/// fields is present.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rate {
    pub value: f32,
}

impl Field for Rate {
    fn from_bytes(input: &[u8]) -> Result<Rate> {
        let value = (Cursor::new(input).read_i8()? as f32) / 2.0;
        Ok(Rate { value })
    }
}

/// The transmitted or received frequency in MHz, including flags describing the
/// channel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Channel {
    /// The frequency in MHz.
    pub freq: u16,
    // The channel flags.
    pub flags: ChannelFlags,
}

impl Field for Channel {
    fn from_bytes(input: &[u8]) -> Result<Channel> {
        let mut cursor = Cursor::new(input);
        let freq = cursor.read_u16::<LE>()?;
        let flags = cursor.read_u16::<LE>()?;
        let flags = ChannelFlags {
            turbo: flags.is_flag_set(0x0010),
            cck: flags.is_flag_set(0x0020),
            ofdm: flags.is_flag_set(0x0040),
            ghz2: flags.is_flag_set(0x0080),
            ghz5: flags.is_flag_set(0x0100),
            passive: flags.is_flag_set(0x0200),
            dynamic: flags.is_flag_set(0x0400),
            gfsk: flags.is_flag_set(0x0800),
        };
        Ok(Channel { freq, flags })
    }
}

/// The hop set and pattern for frequency-hopping radios.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FHSS {
    pub hopset: u8,
    pub pattern: u8,
}

impl Field for FHSS {
    fn from_bytes(input: &[u8]) -> Result<FHSS> {
        let mut cursor = Cursor::new(input);
        let hopset = cursor.read_u8()?;
        let pattern = cursor.read_u8()?;
        Ok(FHSS { hopset, pattern })
    }
}

/// RF signal power at the antenna in dBm. Indicates the RF signal power at the
/// antenna, in decibels difference from 1mW.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AntennaSignal {
    pub value: i8,
}

impl Field for AntennaSignal {
    fn from_bytes(input: &[u8]) -> Result<AntennaSignal> {
        let value = Cursor::new(input).read_i8()?;
        Ok(AntennaSignal { value })
    }
}

/// RF signal power at the antenna in dB. Indicates the RF signal power at the
/// antenna, in decibels difference from an arbitrary, fixed reference.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AntennaSignalDb {
    pub value: u8,
}

impl Field for AntennaSignalDb {
    fn from_bytes(input: &[u8]) -> Result<AntennaSignalDb> {
        let value = Cursor::new(input).read_u8()?;
        Ok(AntennaSignalDb { value })
    }
}

/// RF noise power at the antenna in dBm. Indicates the RF signal noise at the
/// antenna, in decibels  difference from 1mW.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AntennaNoise {
    pub value: i8,
}

impl Field for AntennaNoise {
    fn from_bytes(input: &[u8]) -> Result<AntennaNoise> {
        let value = Cursor::new(input).read_i8()?;
        Ok(AntennaNoise { value })
    }
}

/// RF noise power at the antenna in dB. Indicates the RF signal noise at the
/// antenna, in decibels difference from an arbitrary, fixed reference.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AntennaNoiseDb {
    pub value: u8,
}

impl Field for AntennaNoiseDb {
    fn from_bytes(input: &[u8]) -> Result<AntennaNoiseDb> {
        let value = Cursor::new(input).read_u8()?;
        Ok(AntennaNoiseDb { value })
    }
}

/// Quality of Barker code lock, unitless. Monotonically nondecreasing with
/// "better" lock strength. Called "Signal Quality" in datasheets.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LockQuality {
    pub value: u16,
}

impl Field for LockQuality {
    fn from_bytes(input: &[u8]) -> Result<LockQuality> {
        let value = Cursor::new(input).read_u16::<LE>()?;
        Ok(LockQuality { value })
    }
}

/// Transmit power expressed as unitless distance from max power. 0 is max
/// power. Monotonically nondecreasing with lower power levels.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TxAttenuation {
    pub value: u16,
}

impl Field for TxAttenuation {
    fn from_bytes(input: &[u8]) -> Result<TxAttenuation> {
        let value = Cursor::new(input).read_u16::<LE>()?;
        Ok(TxAttenuation { value })
    }
}

/// Transmit power in dB. 0 is max power. Monotonically nondecreasing with lower
/// power levels.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TxAttenuationDb {
    pub value: u16,
}

impl Field for TxAttenuationDb {
    fn from_bytes(input: &[u8]) -> Result<TxAttenuationDb> {
        let value = Cursor::new(input).read_u16::<LE>()?;
        Ok(TxAttenuationDb { value })
    }
}

/// Transmit power in dBm. This is the absolute power level measured at the
/// antenna port.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TxPower {
    pub value: i8,
}

impl Field for TxPower {
    fn from_bytes(input: &[u8]) -> Result<TxPower> {
        let value = Cursor::new(input).read_i8()?;
        Ok(TxPower { value })
    }
}

/// Indication of the transmit/receive antenna for this frame. The first antenna
/// is antenna 0.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Antenna {
    pub value: u8,
}

impl Field for Antenna {
    fn from_bytes(input: &[u8]) -> Result<Antenna> {
        let value = Cursor::new(input).read_u8()?;
        Ok(Antenna { value })
    }
}

/// Properties of received frames.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RxFlags {
    pub bad_plcp: bool,
}

impl Field for RxFlags {
    fn from_bytes(input: &[u8]) -> Result<RxFlags> {
        let flags = Cursor::new(input).read_u16::<LE>()?;
        Ok(RxFlags {
            bad_plcp: flags.is_flag_set(0x0002),
        })
    }
}

/// Properties of transmitted frames.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TxFlags {
    /// Transmission failed due to excessive retries.
    pub fail: bool,
    /// Transmission used CTS-to-self protection.
    pub cts: bool,
    /// Transmission used RTS/CTS handshake.
    pub rts: bool,
    /// Transmission shall not expect an ACK frame and not retry when no ACK is
    /// received.
    pub no_ack: bool,
    /// Transmission includes a pre-configured sequence number that should not
    /// be changed by the driver's TX handlers.
    pub no_seq: bool,
}

impl Field for TxFlags {
    fn from_bytes(input: &[u8]) -> Result<TxFlags> {
        let flags = Cursor::new(input).read_u8()?;
        Ok(TxFlags {
            fail: flags.is_flag_set(0x0001),
            cts: flags.is_flag_set(0x0002),
            rts: flags.is_flag_set(0x0004),
            no_ack: flags.is_flag_set(0x0008),
            no_seq: flags.is_flag_set(0x0010),
        })
    }
}

/// Number of RTS retries a transmitted frame used.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RTSRetries {
    pub value: u8,
}

impl Field for RTSRetries {
    fn from_bytes(input: &[u8]) -> Result<RTSRetries> {
        let value = Cursor::new(input).read_u8()?;
        Ok(RTSRetries { value })
    }
}

/// Number of data retries a transmitted frame used.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DataRetries {
    pub value: u8,
}

impl Field for DataRetries {
    fn from_bytes(input: &[u8]) -> Result<DataRetries> {
        let value = Cursor::new(input).read_u8()?;
        Ok(DataRetries { value })
    }
}

/// Extended channel information.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct XChannel {
    /// The channel flags.
    pub flags: XChannelFlags,
    /// The frequency in MHz.
    pub freq: u16,
    /// The channel number.
    pub channel: u8,
    /// The max power.
    pub max_power: u8,
}

impl Field for XChannel {
    fn from_bytes(input: &[u8]) -> Result<XChannel> {
        let mut cursor = Cursor::new(input);
        let flags = cursor.read_u32::<LE>()?;
        let freq = cursor.read_u16::<LE>()?;
        let channel = cursor.read_u8()?;
        let max_power = cursor.read_u8()?;
        Ok(XChannel {
            flags: XChannelFlags {
                turbo: flags.is_flag_set(0x00000010),
                cck: flags.is_flag_set(0x00000020),
                ofdm: flags.is_flag_set(0x00000040),
                ghz2: flags.is_flag_set(0x00000080),
                ghz5: flags.is_flag_set(0x00000100),
                passive: flags.is_flag_set(0x00000200),
                dynamic: flags.is_flag_set(0x00000400),
                gfsk: flags.is_flag_set(0x00000800),
                gsm: flags.is_flag_set(0x00001000),
                sturbo: flags.is_flag_set(0x00002000),
                half: flags.is_flag_set(0x00004000),
                quarter: flags.is_flag_set(0x00008000),
                ht20: flags.is_flag_set(0x00010000),
                ht40u: flags.is_flag_set(0x00020000),
                ht40d: flags.is_flag_set(0x00040000),
            },
            freq,
            channel,
            max_power,
        })
    }
}

/// The IEEE 802.11n data rate index. Usually only one of the
/// [Rate](struct.Rate.html), [MCS](struct.MCS.html), and [VHT] fields is
/// present.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MCS {
    /// The bandwidth.
    pub bw: Option<Bandwidth>,
    /// The 802.11n MCS index.
    pub index: Option<u8>,
    /// The guard interval.
    pub gi: Option<GuardInterval>,
    /// The HT format.
    pub format: Option<HTFormat>,
    /// The FEC type.
    pub fec: Option<FEC>,
    /// Number of STBC streams.
    pub stbc: Option<u8>,
    /// Number of extension spatial streams.
    pub ness: Option<u8>,
    /// The datarate in Mbps
    pub datarate: Option<f32>,
}

impl Field for MCS {
    fn from_bytes(input: &[u8]) -> Result<MCS> {
        let mut cursor = Cursor::new(input);
        let mut mcs = MCS {
            ..Default::default()
        };

        let known = cursor.read_u8()?;
        let flags = cursor.read_u8()?;
        let index = cursor.read_u8()?;

        if known.is_flag_set(0x01) {
            mcs.bw = Some(Bandwidth::new(flags & 0x03)?)
        }

        if known.is_flag_set(0x02) {
            mcs.index = Some(index);
        }

        if known.is_flag_set(0x04) {
            mcs.gi = Some(match flags.is_flag_set(0x04) {
                true => GuardInterval::Short,
                false => GuardInterval::Long,
            })
        }

        if known.is_flag_set(0x08) {
            mcs.format = Some(match flags.is_flag_set(0x08) {
                true => HTFormat::Greenfield,
                false => HTFormat::Mixed,
            });
        }

        if known.is_flag_set(0x10) {
            mcs.fec = Some(match flags.is_flag_set(0x10) {
                true => FEC::LDPC,
                false => FEC::BCC,
            });
        }

        if known.is_flag_set(0x20) {
            mcs.stbc = Some(flags.bits_as_int(5, 2));
        }

        if known.is_flag_set(0x40) {
            // Yes this is stored weirdly
            mcs.ness = Some(known & 0x80 >> 6 | flags & 0x80 >> 7)
        }

        if mcs.bw.is_some() && mcs.gi.is_some() {
            mcs.datarate = Some(ht_rate(index, mcs.bw.unwrap(), mcs.gi.unwrap())?);
        }

        Ok(mcs)
    }
}

/// The presence of this field indicates that the frame was received as part of
/// an a-MPDU.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AMPDUStatus {
    /// The A-MPDU reference number.
    pub reference: u32,
    /// Whether this is a 0-length subframe.
    pub zero_length: Option<bool>,
    /// Whether this is the last subframe of this A-MPDU.
    pub last: Option<bool>,
    /// The A-MPDU subframe delimiter CRC.
    pub delimiter_crc: Option<u8>,
}

impl Field for AMPDUStatus {
    fn from_bytes(input: &[u8]) -> Result<AMPDUStatus> {
        let mut cursor = Cursor::new(input);
        let mut ampdu = AMPDUStatus {
            ..Default::default()
        };

        ampdu.reference = cursor.read_u32::<LE>()?;
        let flags = cursor.read_u16::<LE>()?;
        let delim_crc = cursor.read_u8()?;

        if flags.is_flag_set(0x0001) {
            ampdu.zero_length = Some(flags.is_flag_set(0x0002));
        }

        if flags.is_flag_set(0x0004) {
            ampdu.last = Some(flags.is_flag_set(0x0008));
        }

        if !flags.is_flag_set(0x0010) && flags.is_flag_set(0x0020) {
            ampdu.delimiter_crc = Some(delim_crc);
        }

        Ok(ampdu)
    }
}

/// The IEEE 802.11ac data rate index. Usually only one of the
/// [Rate](struct.Rate.html), [MCS](struct.MCS.html), and [VHT](struct.VHT.html)
/// fields is present.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VHT {
    /// Whether all spatial streams of all users have STBC.
    pub stbc: Option<bool>,
    /// Whether STAs may not doze during TXOP or transmitter is non-AP.
    pub txop_ps: Option<bool>,
    /// The guard interval.
    pub gi: Option<GuardInterval>,
    /// False if NSYM mod 10 != 9 or short GI not used. True if NSYM mod 10 = 9.
    pub sgi_nsym_da: Option<bool>,
    /// Whether one or more users are using LDPC and the encoding process
    /// resulted in extra OFDM symbol(s).
    pub ldpc_extra: Option<bool>,
    /// The frame was transmitted/received using beamforming.
    pub beamformed: Option<bool>,
    /// The bandwidth.
    pub bw: Option<Bandwidth>,
    /// The Group ID of the frame.
    pub group_id: Option<u8>,
    /// A non-unique identifier of a STA to identify whether the transmissions
    /// are destined to a STA or not, used in conjunction with GroupID.
    pub partial_aid: Option<u16>,
    /// The users for the current group.
    pub users: [Option<VHTUser>; 4],
}

impl Field for VHT {
    fn from_bytes(input: &[u8]) -> Result<VHT> {
        let mut cursor = Cursor::new(input);
        let mut vht = VHT {
            ..Default::default()
        };

        let known = cursor.read_u16::<LE>()?;
        let flags = cursor.read_u8()?;
        let bandwidth = cursor.read_u8()?;
        let mut mcs_nss = [0; 4];
        cursor.read(&mut mcs_nss)?;
        let coding = cursor.read_u8()?;
        let group_id = cursor.read_u8()?;
        let partial_aid = cursor.read_u16::<LE>()?;

        if known.is_flag_set(0x0001) {
            vht.stbc = Some(flags.is_flag_set(0x01));
        }

        if known.is_flag_set(0x0002) {
            vht.txop_ps = Some(flags.is_flag_set(0x02));
        }

        if known.is_flag_set(0x0004) {
            vht.gi = Some(match flags & 0x04 > 0 {
                true => GuardInterval::Short,
                false => GuardInterval::Long,
            })
        }

        if known.is_flag_set(0x0008) {
            vht.sgi_nsym_da = Some(flags.is_flag_set(0x08));
        }

        if known.is_flag_set(0x0010) {
            vht.ldpc_extra = Some(flags.is_flag_set(0x10));
        }

        if known.is_flag_set(0x0020) {
            vht.beamformed = Some(flags.is_flag_set(0x20));
        }

        if known.is_flag_set(0x0040) {
            vht.bw = Some(Bandwidth::new(bandwidth & 0x1f)?)
        }

        if known.is_flag_set(0x0080) {
            vht.group_id = Some(group_id);
        }

        if known.is_flag_set(0x0100) {
            vht.partial_aid = Some(partial_aid);
        }

        for (i, user) in mcs_nss.iter().enumerate() {
            let nss = user & 0x0f;

            if nss == 0 {
                continue;
            }

            let index = (user & 0xf0) >> 4;
            let nsts = nss << (flags & 0x01);
            let id = i as u8;

            let datarate = if vht.bw.is_some() && vht.gi.is_some() {
                Some(vht_rate(index, vht.bw.unwrap(), vht.gi.unwrap(), nss)?)
            } else {
                None
            };

            vht.users[id as usize] = Some(VHTUser {
                index,
                fec: match (coding & 2 ^ id) >> id {
                    1 => FEC::LDPC,
                    _ => FEC::BCC,
                },
                nss,
                nsts,
                datarate,
            });
        }

        Ok(vht)
    }
}

/// The time the frame was transmitted or received.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Timestamp {
    /// The actual timestamp.
    pub timestamp: u64,
    /// The unit of the timestamp value.
    pub unit: TimeUnit,
    /// The sampling position of the timestamp.
    pub position: SamplingPosition,
    /// The accuracy of the timestamp.
    pub accuracy: Option<u16>,
}

impl Field for Timestamp {
    fn from_bytes(input: &[u8]) -> Result<Timestamp> {
        let mut cursor = Cursor::new(input);

        let timestamp = cursor.read_u64::<LE>()?;
        let mut accuracy = Some(cursor.read_u16::<LE>()?);
        let unit_position = cursor.read_u8()?;
        let unit = TimeUnit::new(unit_position & 0x0f)?;
        let position = SamplingPosition::from(unit_position & 0xf0 >> 4)?;
        let flags = cursor.read_u8()?;

        if !flags.is_flag_set(0x02) {
            accuracy = None;
        }

        Ok(Timestamp {
            timestamp,
            unit,
            position,
            accuracy,
        })
    }
}
