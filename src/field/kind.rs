impl_kind! {
    /// The type of radiotap field.
    #[derive(Debug, Clone, PartialEq)]
    #[non_exhaustive]
    pub enum Kind {
        Tsft            { bit:  0, align: 8, size:  8 },
        Flags           { bit:  1, align: 1, size:  1 },
        Rate            { bit:  2, align: 1, size:  1 },
        Channel         { bit:  3, align: 2, size:  4 },
        Fhss            { bit:  4, align: 2, size:  2 },
        AntennaSignal   { bit:  5, align: 1, size:  1 },
        AntennaNoise    { bit:  6, align: 1, size:  1 },
        LockQuality     { bit:  7, align: 2, size:  2 },
        TxAttenuation   { bit:  8, align: 2, size:  2 },
        TxAttenuationDb { bit:  9, align: 2, size:  2 },
        TxPower         { bit: 10, align: 1, size:  1 },
        Antenna         { bit: 11, align: 1, size:  1 },
        AntennaSignalDb { bit: 12, align: 1, size:  1 },
        AntennaNoiseDb  { bit: 13, align: 1, size:  1 },
        RxFlags         { bit: 14, align: 2, size:  2 },
        RtsRetries      { bit: 16, align: 1, size:  1 },
        TxFlags         { bit: 15, align: 2, size:  2 },
        DataRetries     { bit: 17, align: 1, size:  1 },
        XChannel        { bit: 18, align: 4, size:  8 },
        Mcs             { bit: 19, align: 1, size:  3 },
        AmpduStatus     { bit: 20, align: 4, size:  8 },
        Vht             { bit: 21, align: 2, size: 12 },
        Timestamp       { bit: 22, align: 8, size: 12 },
        He              { bit: 23, align: 8, size: 12 },
        HeMu            { bit: 24, align: 8, size: 12 },
        HeMuUser        { bit: 25, align: 8, size: 12 },
        ZeroLenPsdu     { bit: 26, align: 1, size:  1 },
        LSig            { bit: 27, align: 2, size:  4 },
    }
}
