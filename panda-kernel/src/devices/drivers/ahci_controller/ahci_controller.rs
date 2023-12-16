use x86_64::{PhysAddr, VirtAddr};

use crate::{
    memory,
    pci::{PciDevice, PciRegister},
};

use super::{
    ahci_port::AhciPort,
    ahci_register::AhciRegister,
    registers::{
        AhciBiosOsControlStatusRegister, AhciGlobalHbaControlRegister, AhciHbaCapabilitiesRegister,
    },
};

#[derive(Clone)]
pub struct AhciController {
    pci_device: PciDevice,
    ahci_base: VirtAddr,
}

impl AhciController {
    pub fn new(pci_device: PciDevice) -> Self {
        let ahci_base_phys = pci_device.read::<u32>(PciRegister::BaseAddress5) as u64;
        let ahci_base = memory::physical_to_virtual(PhysAddr::new(ahci_base_phys));

        let ahci_end_phys = ahci_base_phys + 0x1100;
        let ahci_end = memory::physical_to_virtual(PhysAddr::new(ahci_end_phys));

        unsafe {
            memory::mark_as_uncacheable(ahci_base, ahci_end);
        }
        log::info!("    -> Device detection init");

        Self {
            pci_device,
            ahci_base,
        }
    }

    pub fn ahci_base_addr(&self) -> VirtAddr {
        self.ahci_base
    }

    pub fn read(&self, register: AhciRegister) -> u32 {
        let offset = register.offset();
        let addr = self.ahci_base_addr() + offset;
        let value = unsafe { *addr.as_ptr() };
        value
    }

    pub fn write(&self, register: AhciRegister, value: u32) {
        let offset = register.offset();
        let addr = self.ahci_base_addr() + offset;
        unsafe { *addr.as_mut_ptr() = value }
    }

    pub fn port_implemented(&self, index: u8) -> bool {
        let pi = self.read(AhciRegister::PortImplemented);
        pi & (1 << index) != 0
    }

    pub fn port(&self, index: u8) -> Option<AhciPort> {
        if !self.port_implemented(index) {
            return None;
        }

        Some(AhciPort::new(self, index))
    }

    pub(crate) fn pci_device(&mut self) -> &PciDevice {
        &mut self.pci_device
    }

    fn bohc(&self) -> AhciBiosOsControlStatusRegister {
        AhciBiosOsControlStatusRegister(self.read(AhciRegister::BIOSOSHandoffControlAndStatus))
    }

    fn set_bohc(&mut self, bohc: AhciBiosOsControlStatusRegister) {
        self.write(AhciRegister::BIOSOSHandoffControlAndStatus, bohc.0)
    }

    fn ghc(&self) -> AhciGlobalHbaControlRegister {
        AhciGlobalHbaControlRegister(self.read(AhciRegister::GlobalHostControl))
    }

    fn set_ghc(&mut self, ghc: AhciGlobalHbaControlRegister) {
        self.write(AhciRegister::GlobalHostControl, ghc.0)
    }

    pub(crate) fn perform_bios_os_handoff(&mut self) {
        log::info!("AHCI BIOS/OS handoff:");
        let mut bohc = self.bohc();
        if bohc.bos() == false {
            log::info!(" -> BIOS already doesn't have ownership");
            return;
        }

        // assert control
        bohc.set_oos(true);
        self.set_bohc(bohc);

        loop {
            if self.bohc().bos() == false {
                break;
            }
        }

        log::info!("  -> BIOS has cleared its ownership bit");
        log::info!("  -> TODO: wait for BIOS to set BIOS busy bit");
        log::info!("BIOS/OS handoff done");
    }

    pub(crate) fn hba_reset(&mut self) {
        log::info!("Resetting AHCI controller...");

        // set GHC.HR to 1
        let mut ghc = self.ghc();
        ghc.set_hba_reset(true);
        self.set_ghc(ghc);

        // poll waiting for GHC.HR to go to 0
        loop {
            if self.ghc().hba_reset() == false {
                break;
            }
        }

        log::info!("AHCI controller reset complete");
    }

    pub(crate) fn enable_ahci_and_interrupts(&mut self) {
        let mut ghc = self.ghc();
        ghc.set_ahci_enable(true);
        ghc.set_interrupt_enable(true);
        self.set_ghc(ghc);
    }

    pub(crate) fn capabilities(&self) -> AhciHbaCapabilitiesRegister {
        AhciHbaCapabilitiesRegister(self.read(AhciRegister::HostCapability))
    }
}
