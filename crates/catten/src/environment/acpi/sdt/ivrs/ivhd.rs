use crate::memory::PhysicalAddress;

pub(super) struct IommuInfo(u16);

pub(super) struct IommuFeatureReporting(u32);

pub(super) struct Ivhd10H {
   _type: u8,
   flags: u8,
   length: u16,
   device_id: u16,
   capability_offset: u16,
   iommu_base_address: PhysicalAddress,
   pci_segment_group: u16,
   iommu_info: IommuInfo,
   iommu_feature_reporting: IommuFeatureReporting,
}
