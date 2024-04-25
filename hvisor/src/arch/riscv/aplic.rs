
use spin::RwLock;
use spin::Once;
use crate::{cpu::ArchCpu, memory::GuestPhysAddr};
use riscv_decode::Instruction;

// S-mode interrupt delivery controller
const APLIC_S_IDC: usize = 0xd00_4000;
pub const APLIC_DOMAINCFG_BASE: usize = 0x0000;
pub const APLIC_SOURCECFG_BASE: usize = 0x0004;
pub const APLIC_SOURCECFG_TOP: usize = 0x1000;
pub const APLIC_MSIADDR_BASE: usize = 0x1BC8;
pub const APLIC_PENDING_BASE: usize = 0x1C00;
pub const APLIC_PENDING_TOP: usize = 0x1C80;
pub const APLIC_CLRIP_BASE: usize = 0x1D00;
pub const APLIC_ENABLE_BASE: usize = 0x1E00;
pub const APLIC_ENABLE_TOP: usize = 0x1E7C;
pub const APLIC_ENABLE_NUM: usize = 0x1EDC;
pub const APLIC_CLRIE_BASE: usize = 0x1F00;
pub const APLIC_TARGET_BASE: usize = 0x3004;
pub const APLIC_IDC_BASE: usize = 0x4000;

#[repr(u32)]
#[allow(dead_code)]
pub enum SourceModes { 
    Inactive = 0,
    Detached = 1,
    RisingEdge = 4,
    FallingEdge = 5,
    LevelHigh = 6,
    LevelLow = 7,
}

// offset size register name
// 0x0000 4 bytes domaincfg
// 0x0004 4 bytes sourcecfg[1]
// 0x0008 4 bytes sourcecfg[2]
// . . . . . .
// 0x0FFC 4 bytes sourcecfg[1023]
// 0x1BC0 4 bytes mmsiaddrcfg (machine-level interrupt domains only)
// 0x1BC4 4 bytes mmsiaddrcfgh ”
// 0x1BC8 4 bytes smsiaddrcfg ”
// 0x1BCC 4 bytes smsiaddrcfgh ”
// 0x1C00 4 bytes setip[0]
// 0x1C04 4 bytes setip[1]
// . . . . . .
// 0x1C7C 4 bytes setip[31]
// 0x1CDC 4 bytes setipnum
// 0x1D00 4 bytes in clrip[0]
// 0x1D04 4 bytes in clrip[1]
// . . . . . .
// 0x1D7C 4 bytes in clrip[31]
// 0x1DDC 4 bytes clripnum
// 0x1E00 4 bytes setie[0]
// 0x1E04 4 bytes setie[1]
// . . . . . .
// 0x1E7C 4 bytes setie[31]
// 0x1EDC 4 bytes setienum
// 0x1F00 4 bytes clrie[0]
// 0x1F04 4 bytes clrie[1]
// . . . . . .
// 0x1F7C 4 bytes clrie[31]
// 0x1FDC 4 bytes clrienum
// 0x2000 4 bytes setipnum le
// 0x2004 4 bytes setipnum be
// 0x3000 4 bytes genmsi
// 0x3004 4 bytes target[1]
// 0x3008 4 bytes target[2]
// . . . . . .
// 0x3FFC 4 bytes target[1023]


pub static APLIC: Once<RwLock<Aplic>> = Once::new();

pub fn host_aplic<'a>() -> &'a RwLock<Aplic> {
    APLIC.get().expect("Uninitialized hypervisor aplic!")
}
pub fn init_aplic(aplic_base: usize, aplic_size: usize) {
    let aplic = Aplic::new(aplic_base, aplic_size);
    APLIC.call_once(|| RwLock::new(aplic));
}

#[repr(C)]
pub struct Aplic {
    pub base: usize,
    pub size: usize,
}

#[allow(dead_code)]
impl Aplic {
    pub fn new(base: usize, size: usize) -> Self {
        Self {
            base,
            size,
        }
    }
    pub fn set_domaincfg(&self, bigendian: bool, msimode: bool, enabled: bool){
        let enabled = u32::from(enabled);
        let msimode = u32::from(msimode);
        let bigendian = u32::from(bigendian);
        let addr = self.base + APLIC_DOMAINCFG_BASE;
        let src = (enabled << 8) | (msimode << 2) | bigendian;
        unsafe {
            core::ptr::write_volatile(addr as *mut u32, src);
        }
    }
    pub fn read_domaincfg(&self) -> u32{
        let addr = self.base + APLIC_DOMAINCFG_BASE;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    pub fn get_msimode(&self) -> bool{
        let addr = self.base + APLIC_DOMAINCFG_BASE;
        let value= unsafe { core::ptr::read_volatile(addr as *const u32) };
        ((value >> 2) & 0b11) != 0
    }
    pub fn set_sourcecfg(&self, irq: u32, mode: SourceModes){
        assert!(irq > 0 && irq < 1024);
        let addr = self.base + APLIC_SOURCECFG_BASE + (irq as usize - 1) * 4;
        let src = mode as u32;
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, src);
        }
    } 
    pub fn set_sourcecfg_delegate(&self, irq: u32, child: u32){
        assert!(irq > 0 && irq < 1024);
        let addr = self.base + APLIC_SOURCECFG_BASE + (irq as usize - 1) * 4;
        let src = 1 << 10 | child & 0x3ff;
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, src);
        }
    } 
    pub fn read_sourcecfg(&self, irq: u32) -> u32{
        assert!(irq > 0 && irq < 1024);
        let addr = self.base + APLIC_SOURCECFG_BASE + (irq as usize - 1) * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    pub fn set_msiaddr(&self, address: usize){
        let addr = self.base + APLIC_MSIADDR_BASE;
        let src = (address >> 12) as u32;
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, src);
            core:: ptr::write_volatile((addr + 4) as *mut u32, 0);
        }
    }
    // pub fn read_pending_irq(&self, irq: u32) -> u32{
    //     assert!(irq > 0 && irq < 1024);
    //     let irqidx = irq as usize / 32;
    //     let addr = self.base + APLIC_PENDING_BASE + irqidx * 4;
    //     unsafe { core::ptr::read_volatile(addr as *const u32) }
    // }
    pub fn read_pending(&self, irqidx: usize) -> u32{
        assert!(irqidx < 32);
        let addr = self.base + APLIC_PENDING_BASE + irqidx * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    // pub fn read_clr_pending_irq(&self, irq: u32) -> u32{
    //     assert!(irq > 0 && irq < 1024);
    //     let irqidx = irq as usize / 32;
    //     let addr = self.base + APLIC_CLRIP_BASE + irqidx * 4;
    //     unsafe { core::ptr::read_volatile(addr as *const u32) }
    // }
    pub fn read_clr_pending(&self, irqidx: usize) -> u32{
        assert!(irqidx < 32);
        let addr = self.base + APLIC_CLRIP_BASE + irqidx * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    // pub fn set_pending_irq(&self, irq: u32, pending: bool){
    //     assert!(irq > 0 && irq < 1024);
    //     let irqidx = irq as usize / 32;
    //     let irqbit = irq as usize % 32;
    //     let addr = self.base + APLIC_PENDING_BASE + irqidx * 4;
    //     let clr_addr = self.base + APLIC_CLRIP_BASE + irqidx * 4;
    //     let src = 1 << irqbit;
    //     if pending {
    //         unsafe{
    //             core:: ptr::write_volatile(addr as *mut u32, src);
    //         }
    //     } else {
    //         unsafe{
    //             core:: ptr::write_volatile(clr_addr as *mut u32, src);
    //         }
    //     }
    // } 
    pub fn set_pending(&self, irqidx: usize, value: u32, pending: bool){
        assert!(irqidx < 32);
        let addr = self.base + APLIC_PENDING_BASE + irqidx * 4;
        let clr_addr = self.base + APLIC_CLRIP_BASE + irqidx * 4;
        if pending {
            unsafe{
                core:: ptr::write_volatile(addr as *mut u32, value);
            }
        } else {
            unsafe{
                core:: ptr::write_volatile(clr_addr as *mut u32, value);
            }
        }
    } 
    // pub fn read_enable_irq(&self, irq: u32) -> u32{
    //     assert!(irq > 0 && irq < 1024);
    //     let irqidx = irq as usize / 32;
    //     let addr = self.base + APLIC_ENABLE_BASE + irqidx * 4;
    //     unsafe { core::ptr::read_volatile(addr as *const u32) }
    // }
    pub fn read_enable(&self, irqidx: usize) -> u32{
        assert!(irqidx < 32);
        let addr = self.base + APLIC_ENABLE_BASE + irqidx * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    // pub fn read_clr_enable_irq(&self, irq: u32) -> u32{
    //     assert!(irq > 0 && irq < 1024);
    //     let irqidx = irq as usize / 32;
    //     let addr = self.base + APLIC_CLRIE_BASE + irqidx * 4;
    //     unsafe { core::ptr::read_volatile(addr as *const u32) }
    // }
    pub fn read_clr_enable(&self, irqidx: usize) -> u32{
        assert!(irqidx < 32);
        let addr = self.base + APLIC_CLRIE_BASE + irqidx * 4;
        unsafe { core::ptr::read_volatile(addr as *const u32) }
    }
    // pub fn set_enable_irq(&self, irq: u32, enabled: bool){
    //     assert!(irq > 0 && irq < 1024);
    //     let irqidx = irq as usize / 32;
    //     let irqbit = irq as usize % 32;
    //     let addr = self.base + APLIC_ENABLE_BASE + irqidx * 4;
    //     let clr_addr = self.base + APLIC_CLRIE_BASE + irqidx * 4;
    //     let src = 1 << irqbit;
    //     if enabled {
    //         unsafe{
    //             core:: ptr::write_volatile(addr as *mut u32, src);
    //         }
    //     } else {
    //         unsafe{
    //             core:: ptr::write_volatile(clr_addr as *mut u32, src);
    //         }
    //     }
    // } 
    pub fn set_enable(&self, irqidx: usize, value: u32, enabled: bool){
        assert!(irqidx < 32);
        let addr = self.base + APLIC_ENABLE_BASE + irqidx * 4;
        let clr_addr = self.base + APLIC_CLRIE_BASE + irqidx * 4;
        if enabled {
            unsafe{
                core:: ptr::write_volatile(addr as *mut u32, value);
            }
        } else {
            unsafe{
                core:: ptr::write_volatile(clr_addr as *mut u32, value);
            }
        }
    } 
    pub fn set_enable_num(&self, value: u32){
        let addr = self.base + APLIC_ENABLE_NUM;
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, value);
        }
    }
    pub fn set_target_msi(&self, irq: u32, hart: u32, guest: u32, eiid: u32){
        let addr = self.base + APLIC_TARGET_BASE + (irq as usize - 1) * 4;
        let src = (hart << 18) | (guest << 12) | eiid;
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, src);
        }
    }
    pub fn set_target_direct(&self, irq: u32, hart: u32, prio: u32){
        let addr = self.base + APLIC_TARGET_BASE + (irq as usize - 1) * 4;
        let src =  (hart << 18) | (prio & 0xFF);
        unsafe{
            core:: ptr::write_volatile(addr as *mut u32, src);
        }
    }
}
/// Interrupt Delivery Control is only used in 'direct' mode
#[repr(C)]
struct InterruptDeliveryControl {
    pub idelivery: u32,
    pub iforce: u32,
    pub ithreshold: u32,
    pub topi: u32,
    pub claimi: u32,
}

#[allow(dead_code)]
impl InterruptDeliveryControl {
    const fn ptr(hart: usize) -> *mut Self {
        assert!(hart < 1024);
        (APLIC_S_IDC + hart * 32) as *mut Self
    }

    pub fn as_ref<'a>(hart: usize) -> &'a Self {
        unsafe { Self::ptr(hart).as_ref().unwrap() }
    }

    pub fn as_mut<'a>(hart: usize) -> &'a mut Self {
        unsafe { Self::ptr(hart).as_mut().unwrap() }
    }
}

pub fn vaplic_emul_handler(
    current_cpu: &mut ArchCpu,
    addr: GuestPhysAddr,
    inst: Instruction,
) {
    let host_aplic = host_aplic();
    let offset = addr.wrapping_sub(host_aplic.read().base);
    if offset >= APLIC_DOMAINCFG_BASE && offset < APLIC_SOURCECFG_BASE {
        match inst {
            Instruction::Sw(i) => {
                let value = current_cpu.x[i.rs2() as usize] as u32;      // 要写入的 value
                let enabled = ((value >> 8) & 0x1) != 0;                // IE
                let msimode = ((value >> 2) & 0b1) != 0;                // DM / MSI
                let bigendian = (value & 0b1) != 0;                     // 大小端
                host_aplic.write().set_domaincfg(bigendian, msimode, enabled);
                info!(
                    "APLIC set domaincfg write addr@{:#x} bigendian {} msimode {} enabled {}",
                    addr, bigendian, msimode, enabled
                );
            }
            Instruction::Lw(i) => {                                     // 直接读取对应的内容
                let value = host_aplic.read().read_domaincfg();
                current_cpu.x[i.rd() as usize] = value as usize;
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    }  
    else if offset >= APLIC_SOURCECFG_BASE && offset < APLIC_SOURCECFG_TOP {
        //sourcecfg
        match inst {
            Instruction::Sw(i) => {
                let value = current_cpu.x[i.rs2() as usize] as u32;
                let irq = ((offset - APLIC_SOURCECFG_BASE) / 4) + 1;
                if (value >> 10) & 0b1 == 1 {
                    //delegate
                    let child = value & 0x3ff;
                    host_aplic.write().set_sourcecfg_delegate(irq as u32, child);
                    info!(
                        "APLIC set sourcecfg_delegate write addr@{:#x} irq {} child {}",
                        addr,
                        irq,
                        child
                    );
                } else {    
                    let mode = match value {
                        0 => SourceModes::Inactive,
                        1 => SourceModes::Detached,
                        4 => SourceModes::RisingEdge,
                        5 => SourceModes::FallingEdge,
                        6 => SourceModes::LevelHigh,
                        7 => SourceModes::LevelLow,
                        _ => panic!("Unknown sourcecfg mode"),
                    };
                    host_aplic.write().set_sourcecfg(irq as u32, mode);
                    info!(
                        "APLIC set sourcecfg write addr@{:#x} irq {} mode {}",
                        addr,
                        irq,
                        value
                    );
                }
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    } else if offset >= APLIC_MSIADDR_BASE && offset <= 0x1BCC {
        // msia
         match inst {
            Instruction::Sw(i) => {
                let value = current_cpu.x[i.rs2() as usize] as u32;
                let address = (value as usize) << 12;
                host_aplic.write().set_msiaddr(address);
                info!(
                    "APLIC set msiaddr write addr@{:#x} address {}",
                    addr, address
                );
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    } else if offset >= APLIC_PENDING_BASE && offset < APLIC_PENDING_TOP {
        // pending
        panic!("setip Unexpected instruction {:?}", inst);
    } 
    // setipnum 区域        0x1CDC  -  0x1CE0
    else if offset >= 0x1CDC && offset < 0x1CE0 {
        panic!("setipnum Unexpected instruction {:?}", inst)
    }
    else if offset >= APLIC_CLRIP_BASE && offset < 0x1D80 {
        panic!("in_clrip Unexpected instruction {:?}", inst);
    }
    // clripnum 区域
    else if offset >= 0x1DDC && offset < 0x1DE0 {
        panic!("clripnum Unexpected instruction {:?}", inst)
    }
    // setie
    else if offset >= APLIC_ENABLE_BASE && offset < 0x1E80 {
        panic!("setie Unexpected instruction {:?}", inst);
    }  
    else if offset >= APLIC_ENABLE_NUM && offset < 0x1EE0 {
        // enablenum
        match inst {
            Instruction::Sw(i) => {
                let value = current_cpu.x[i.rs2() as usize] as u32;
                host_aplic.write().set_enable_num(value);
                info!(
                    "APLIC set enablenum write addr@{:#x} value {}",
                    addr, value
                );
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    } else if offset >= APLIC_CLRIE_BASE && offset < 0x1FDC {
        // clrenable
        match inst {
            Instruction::Sw(i) => {
                let value = current_cpu.x[i.rs2() as usize] as u32;
                let irqidx = (offset - APLIC_CLRIE_BASE) / 4;
                host_aplic.write().set_enable(irqidx, value, false);
                info!(
                    "APLIC set clr_enable write addr@{:#x} irqidx {} value {}",
                    addr, irqidx, value
                );
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    }  
    // clrienum
    else if offset >= 0x1FDC && offset < 0x1FE0 {
        panic!("clrienum Unexpected instruction {:?}", inst)
    }
    // setipnum_le & setipnum_be
    else if offset >= 0x2000 && offset < 0x2008 {
        panic!("setipnum_le Unexpected instruction {:?}", inst)
    }
    // genmsi 
    else if offset >= 0x3000 && offset < 0x3004 {
        panic!("genmsi Unexpected instruction {:?}", inst)
    }
    else if offset >= APLIC_TARGET_BASE && offset < APLIC_IDC_BASE {
        // target
        match inst {
            Instruction::Sw(i) => {
                let value = current_cpu.x[i.rs2() as usize] as u32;
                let irq = ((offset - APLIC_TARGET_BASE) / 4) as u32 + 1;
                let hart = (value >> 18) & 0x3F;
                if host_aplic.read().get_msimode() {
                    let guest = (value >> 12) & 0x3F;
                    let eiid = value & 0xFFF;
                    host_aplic.write().set_target_msi(irq, hart, 1, eiid);
                    info!(
                        "APLIC set msi target write addr@{:#x} irq {} hart {} guest {} eiid {}",
                        addr, irq, hart, guest, eiid
                    );
                } else {
                    let prio = value & 0xFF;
                    host_aplic.write().set_target_direct(irq, hart, prio);
                    info!(
                        "APLIC set direct target write addr@{:#x} irq {} hart {} prio {}",
                        addr, irq, hart, prio
                    );
                }
            }
            _ => panic!("Unexpected instruction {:?}", inst),
        }
    } else {
        panic!("Invalid address: {:#x}", addr);
    }
}