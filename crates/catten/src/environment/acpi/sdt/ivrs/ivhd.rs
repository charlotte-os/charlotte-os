use crate::klib::bitwise;
use crate::memory::PhysicalAddress;

pub(super) struct FlagsLegacy(u8);

impl FlagsLegacy {
    fn remote_iotlb_supported(&self) -> bool {
        const REMOTE_IOTLB_SUPPORTED_SHIFT: u8 = 4;
        (self.0 >> REMOTE_IOTLB_SUPPORTED_SHIFT) & 1 != 0
    }

    fn prefetch_iommu_pages_supported(&self) -> bool {
        const PREFETCH_IOMMU_PAGES_SUPPORTED_SHIFT: u8 = 6;
        (self.0 >> PREFETCH_IOMMU_PAGES_SUPPORTED_SHIFT) & 1 != 0
    }

    fn peripheral_page_service_supported(&self) -> bool {
        const PERIPHERAL_PAGE_SERVICE_SUPPORTED_SHIFT: u8 = 7;
        (self.0 >> PERIPHERAL_PAGE_SERVICE_SUPPORTED_SHIFT) & 1 != 0
    }
}
pub(super) struct IommuInfo(u16);
impl IommuInfo {
    fn msi_num(&self) -> u8 {
        const MSI_NUM_MASK: u16 = 0b1111;
        (self.0 & MSI_NUM_MASK) as u8
    }

    fn unit_id_num(&self) -> u8 {
        const UNIT_ID_NUM_MASK: u16 = 0b1111;
        const UNIT_ID_NUM_SHIFT: u8 = 4;
        bitwise::read_subfield(self.0, UNIT_ID_NUM_SHIFT, UNIT_ID_NUM_MASK) as u8
    }
}

pub(super) struct IommuFeatureReporting(u32);

impl IommuFeatureReporting {
    fn x2apic_supported(&self) -> bool {
        self.0 & 1 != 0
    }

    fn nx_supported(&self) -> bool {
        (self.0 >> 1) & 1 != 0
    }

    fn guest_tl_supported(&self) -> bool {
        (self.0 >> 2) & 1 != 0
    }

    fn num_guest_cr3_tables_supported(&self) -> bool {
        const NUM_GUEST_CR3_TABLES_SUPPORTED_MASK: u32 = 0b11;
        const NUM_GUEST_CR3_TABLES_SUPPORTED_SHIFT: u8 = 3;
        bitwise::read_subfield(
            self.0,
            NUM_GUEST_CR3_TABLES_SUPPORTED_SHIFT,
            NUM_GUEST_CR3_TABLES_SUPPORTED_MASK,
        ) != 0
    }

    fn inval_iommu_all_supported(&self) -> bool {
        (self.0 >> 5) & 1 != 0
    }

    fn guest_vapic_supported(&self) -> bool {
        (self.0 >> 6) & 1 != 0
    }

    fn hardware_error_regs_supported(&self) -> bool {
        (self.0 >> 7) & 1 != 0
    }

    fn max_pasid_val(&self) -> u8 {
        const MAX_PASID_VAL_MASK: u32 = 0b1111;
        const MAX_PASID_VAL_SHIFT: u8 = 8;
        bitwise::read_subfield(self.0, MAX_PASID_VAL_SHIFT, MAX_PASID_VAL_MASK) as u8
    }

    fn num_perf_counters_per_bank(&self) -> u8 {
        const NUM_PERF_COUNTER_PER_BANK_MASK: u32 = 0b1111;
        const NUM_PERF_COUNTER_PER_BANK_SHIFT: u8 = 13;
        bitwise::read_subfield(
            self.0,
            NUM_PERF_COUNTER_PER_BANK_SHIFT,
            NUM_PERF_COUNTER_PER_BANK_MASK,
        ) as u8
    }

    fn num_perf_counter_banks(&self) -> u8 {
        const NUM_PERF_COUNTER_BANKS_MASK: u32 = 0b1111;
        const NUM_PERF_COUNTER_BANKS_SHIFT: u8 = 17;
        bitwise::read_subfield(self.0, NUM_PERF_COUNTER_BANKS_SHIFT, NUM_PERF_COUNTER_BANKS_MASK)
            as u8
    }

    fn msi_num_ppr(&self) -> u8 {
        const MSI_NUM_PPR_MASK: u32 = (1 << 5) - 1;
        const MSI_NUM_PPR_SHIFT: u8 = 23;
        bitwise::read_subfield(self.0, MSI_NUM_PPR_SHIFT, MSI_NUM_PPR_MASK) as u8
    }

    fn guest_addr_tl_size(&self) -> u8 {
        const GUEST_ADDR_TL_SIZE_MASK: u32 = 0b11;
        const GUEST_ADDR_TL_SIZE_SHIFT: u8 = 28;
        bitwise::read_subfield(self.0, GUEST_ADDR_TL_SIZE_SHIFT, GUEST_ADDR_TL_SIZE_MASK) as u8
    }

    fn host_addr_tl_size(&self) -> u8 {
        const HOST_ADDR_TL_SIZE_MASK: u32 = 0b11;
        const HOST_ADDR_TL_SIZE_SHIFT: u8 = 30;
        bitwise::read_subfield(self.0, HOST_ADDR_TL_SIZE_SHIFT, HOST_ADDR_TL_SIZE_MASK) as u8
    }
}

/// The legacy IVHD structure(Type 10h)
pub(super) struct IvhdLegacy {
    _ivhd_type: u8,
    pub flags: FlagsLegacy,
    pub length: u16,
    pub device_id: u16,
    pub capability_offset: u16,
    pub iommu_base_address: PhysicalAddress,
    pub pci_segment_group: u16,
    pub iommu_info: IommuInfo,
    pub iommu_feature_reporting: IommuFeatureReporting,
}

pub struct FlagsModern(u32);

impl FlagsModern {
    fn remote_iotlb_supported(&self) -> bool {
        const REMOTE_IOTLB_SUPPORTED_SHIFT: u8 = 4;
        (self.0 >> REMOTE_IOTLB_SUPPORTED_SHIFT) & 1 != 0
    }
}

pub struct IommuAttributes(u32);

impl IommuAttributes {
    fn num_perf_counters_per_bank(&self) -> u8 {
        const NUM_PERF_COUNTER_PER_BANK_MASK: u32 = 0b1111;
        const NUM_PERF_COUNTER_PER_BANK_SHIFT: u8 = 13;
        bitwise::read_subfield(
            self.0,
            NUM_PERF_COUNTER_PER_BANK_SHIFT,
            NUM_PERF_COUNTER_PER_BANK_MASK,
        ) as u8
    }

    fn num_perf_counter_banks(&self) -> u8 {
        const NUM_PERF_COUNTER_BANKS_MASK: u32 = 0b1111;
        const NUM_PERF_COUNTER_BANKS_SHIFT: u8 = 17;
        bitwise::read_subfield(self.0, NUM_PERF_COUNTER_BANKS_SHIFT, NUM_PERF_COUNTER_BANKS_MASK)
            as u8
    }

    /// Returns the number of MSI PPRs supported by the IOMMU.
    fn msi_num_ppr(&self) -> u8 {
        const MSI_NUM_PPR_MASK: u32 = (1 << 5) - 1;
        const MSI_NUM_PPR_SHIFT: u8 = 23;
        bitwise::read_subfield(self.0, MSI_NUM_PPR_SHIFT, MSI_NUM_PPR_MASK) as u8
    }

    /// If true then translation is not supported. On physical hardware this should always be false.
    fn host_address_translation_disabled(&self) -> bool {
        self.0 & 1 != 0
    }
}

/// The modern IVHD structure(Type 11h/40h)
pub(super) struct IvhdModern {
    pub ivhd_type: u8,
    pub flags: FlagsModern,
    pub length: u16,
    pub device_id: u16,
    pub capability_offset: u16,
    pub iommu_base_address: PhysicalAddress,
    pub pci_segment_group: u16,
    pub iommu_info: IommuInfo,
    pub iommu_attributes: IommuAttributes,
    pub efr_register_image: u64,
    pub efr_register_image2: u64,
}

impl IvhdModern {
    pub fn get_first_device_entry_ptr(&self) -> *const c_void {
        unsafe { (&raw const *self).byte_add(size_of::<IvhdModern>()) }
    }
}
