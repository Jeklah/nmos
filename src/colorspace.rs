use nmos::string_enum;

// Colorspace (used in video flows)
// See https://specs.amwa.tv/is-04/releases/v1.2.0/APIs/schemas/with-refs/flow_video.html
// and https://specs.amwa.tv/nmos-parameter-registers/branches/main/flow-attributes/#colorspace
string_enum! {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Colorspace {
        // Recommendation ITU-R BT.601-7
        const BT601 = "BT601",
        // Recommendation ITU-R BT.709-6
        const BT709 = "BT709",
        // Recommendation ITU-R BT.2020-2
        const BT2020 = "BT2020",
        // Recommendation ITU-R BT.2100 Table 2 titled "System colorimetry"
        const BT2100 = "BT2100",
        // Since IS-04 v1.3, colorspace values may be defined in the Flow Attributes register of the NMOS Parameter Registers
        // SMPTE ST 2065-1 Academy Color Encoding Specification (ACES)
        const ST2065_1 = "ST2065-1",
        // SMPTE ST 2065-3 Academy Density Exchange Encoding (ADX)
        const ST2065_3 = "ST2065-3",
        // ISO 11664-1 CIE 1931 standard colorimetric system
        const XYZ = "XYZ",
    }
}

