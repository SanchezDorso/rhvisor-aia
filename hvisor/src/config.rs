use spin::Mutex;
pub const DTB_ADDR: usize = 0xbfe00000;
#[link_section = ".dtb"]
/// the guest dtb file
pub static GUEST1_DTB: [u8; include_bytes!("../../guests/linux3.dtb").len()] =
    *include_bytes!("../../guests/linux3.dtb");
#[link_section = ".initrd"]
/// the guest kernel file
pub static GUEST1: [u8; include_bytes!("../../guests/Image-62").len()] =
    *include_bytes!("../../guests/Image-62");

// #[link_section = ".dtb"]
// /// the guest dtb file
// pub static GUEST1_DTB: [u8; include_bytes!("../../guests/linux4.dtb").len()] =
//     *include_bytes!("../../guests/linux4.dtb");
// #[link_section = ".initrd"]
// /// the guest kernel file
// pub static GUEST1: [u8; include_bytes!("../../guests/bao-linux.bin").len()] =
//     *include_bytes!("../../guests/bao-linux.bin");
pub static GUESTS: [(&'static [u8], &'static [u8]); 1] = [(&GUEST1, &GUEST1_DTB)];
// #[link_section = ".dtb"]
// /// the guest dtb file
// pub static GUEST_DTB: [u8; include_bytes!("../../guests/rCore-Tutorial-v3/rCore-Tutorial-v3.dtb")
//     .len()] = *include_bytes!("../../guests/rCore-Tutorial-v3/rCore-Tutorial-v3.dtb");
// #[link_section = ".initrd"]
// static GUEST: [u8; include_bytes!("../../guests/rCore-Tutorial-v3/rCore-Tutorial-v3.bin").len()] =
//     *include_bytes!("../../guests/rCore-Tutorial-v3/rCore-Tutorial-v3.bin");
// #[link_section = ".dtb"]
// /// the guest dtb file
// pub static GUEST2_DTB: [u8; include_bytes!("../../guests/os_ch5.dtb").len()] =
//     *include_bytes!("../../guests/os_ch5.dtb");
// #[link_section = ".rcore"]
// /// the guest kernel file
// pub static GUEST2: [u8; include_bytes!("../../guests/os_ch5_802.bin").len()] =
//     *include_bytes!("../../guests/os_ch5_802.bin");
// // pub static GUESTS: [(&'static [u8], &'static [u8]); 1] = [(&GUEST2, &GUEST2_DTB)];
// pub static GUESTS: [(&'static [u8], &'static [u8]); 2] =
//     [(&GUEST1, &GUEST1_DTB), (&GUEST2, &GUEST2_DTB)];
