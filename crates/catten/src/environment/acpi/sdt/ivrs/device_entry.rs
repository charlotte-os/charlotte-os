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
