use crate::environment::acpi::sdt::ivrs::EntryType;

#[repr(transparent)]
struct DteSetting(u8);

impl DteSetting {
    fn init_capable(&self) -> bool {
        self.0 & 0x1 != 0
    }

    fn ext_int_capable(&self) -> bool {
        (self.0 >> 1) & 1 != 0
    }

    fn nmi_capable(&self) -> bool {
        (self.0 >> 2) & 1 != 0
    }

    fn ats_permitted(&self) -> bool {
        (self.0 >> 3) & 1 != 0
    }
}

#[repr(C)]
struct DeviceEntryFixed4 {
    entry_type: EntryType,
    device_id: u8,
    dte_setting: DteSetting,
}

#[repr(C)]
struct DeviceEntryFixed8 {
    entry_type: EntryType,
    device_id: u8,
    dte_setting: DteSetting,
    extended_bytes: [u8; 4],
}

#[repr(u8)]
enum UidFormat {
    NotPresent = 0,
    Integer = 1,
    String = 2,
}

#[repr(C)]
struct DeviceEntryVariable {
    entry_type: EntryType,
    device_id: u16,
    dte_setting: DteSetting,
    hid: u32,
    cid: u32,
    uid_fmt: UidFormat,
    uid_len: u8,
}
