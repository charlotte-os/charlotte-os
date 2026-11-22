#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
struct DteSetting(u8);

impl DteSetting {
    fn init_capable(self) -> bool {
        self.0 & 0x1 != 0
    }

    fn ext_int_capable(self) -> bool {
        (self.0 >> 1) & 1 != 0
    }

    fn nmi_capable(self) -> bool {
        (self.0 >> 2) & 1 != 0
    }

    fn ats_permitted(self) -> bool {
        (self.0 >> 3) & 1 != 0
    }
}

#[repr(C, packed)]
struct DeviceEntryFixed4 {
    entry_type: u8,
    device_id: u16,
    dte_setting: DteSetting,
}

const _: () = assert!(size_of::<DeviceEntryFixed4>() == 4);

#[repr(C, packed)]
struct DeviceEntryFixed8 {
    entry_type: u8,
    device_id: u16,
    dte_setting: DteSetting,
    extended_bytes: [u8; 4],
}

const _: () = assert!(size_of::<DeviceEntryFixed8>() == 8);

#[repr(u8)]
enum UidFormat {
    NotPresent = 0,
    Integer = 1,
    String = 2,
}

/// ACPI HID device entry (Type F0h). The UID, if present, follows this fixed-size portion.
#[repr(C, packed)]
struct DeviceEntryVariable {
    entry_type: u8,
    device_id: u16,
    dte_setting: DteSetting,
    hid: [u8; 8],
    cid: [u8; 8],
    uid_fmt: u8,
    uid_len: u8,
}

const _: () = assert!(size_of::<DeviceEntryVariable>() == 22);

impl DeviceEntryVariable {
    fn uid_format(&self) -> Option<UidFormat> {
        match self.uid_fmt {
            0 => Some(UidFormat::NotPresent),
            1 => Some(UidFormat::Integer),
            2 => Some(UidFormat::String),
            _ => None,
        }
    }
}
