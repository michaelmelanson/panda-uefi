use bitfield::bitfield;

bitfield! {
    pub struct AhciPhysicalRegionDescriptor([u32]);
    impl Debug;
    u32;

    // DW0
    pub data_base_addr_lower, set_data_base_addr_lower: 31, 0;

    // DW1
    pub data_base_addr_upper, set_data_base_addr_upper: 63, 32;

    // DW2 reserved
    // Bits 95-64

    // DW3
    pub interrupt_on_completion, set_interrupt_on_completion: 127;
    pub data_byte_count, set_data_byte_count: 117, 96;
}

impl<T: Clone> Clone for AhciPhysicalRegionDescriptor<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: Copy> Copy for AhciPhysicalRegionDescriptor<T> {}
