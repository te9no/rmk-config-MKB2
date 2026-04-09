use embassy_nrf::mode::Async;
use embassy_nrf::pac;
use embassy_nrf::rng;
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{self as sdc};
use rmk::matrix::bidirectional_matrix::ScanLocation;

pub const ROW: usize = 5;
pub const COL: usize = 6;
pub const PIN_NUM: usize = 6;

// Electrical lines used by the ZMK charlieplex kscan:
// D0, D1, D2, D3, NFC1, NFC2
// -> P0_02, P0_03, P0_28, P0_29, P0_09, P0_10
pub const SCAN_MAP: [[ScanLocation; COL]; ROW] = [
    [
        ScanLocation::Pins(1, 0),
        ScanLocation::Pins(2, 0),
        ScanLocation::Pins(3, 0),
        ScanLocation::Pins(4, 0),
        ScanLocation::Pins(5, 0),
        ScanLocation::Pins(2, 1),
    ],
    [
        ScanLocation::Pins(0, 2),
        ScanLocation::Pins(1, 2),
        ScanLocation::Pins(3, 2),
        ScanLocation::Pins(4, 2),
        ScanLocation::Pins(5, 2),
        ScanLocation::Pins(3, 1),
    ],
    [
        ScanLocation::Pins(0, 3),
        ScanLocation::Pins(1, 3),
        ScanLocation::Pins(2, 3),
        ScanLocation::Pins(4, 3),
        ScanLocation::Pins(5, 3),
        ScanLocation::Pins(4, 1),
    ],
    [
        ScanLocation::Pins(0, 4),
        ScanLocation::Pins(1, 4),
        ScanLocation::Pins(2, 4),
        ScanLocation::Ignore,
        ScanLocation::Pins(3, 4),
        ScanLocation::Pins(5, 4),
    ],
    [
        ScanLocation::Pins(3, 5),
        ScanLocation::Pins(2, 5),
        ScanLocation::Pins(4, 5),
        ScanLocation::Pins(1, 5),
        ScanLocation::Pins(0, 5),
        ScanLocation::Ignore,
    ],
];

/// How many outgoing L2CAP buffers per link
pub const L2CAP_TXQ: u8 = 3;

/// How many incoming L2CAP buffers per link
pub const L2CAP_RXQ: u8 = 3;

/// Size of L2CAP packets
pub const L2CAP_MTU: usize = 251;

pub fn build_sdc<'d, const N: usize>(
    p: nrf_sdc::Peripherals<'d>,
    rng: &'d mut rng::Rng<Async>,
    mpsl: &'d MultiprotocolServiceLayer,
    mem: &'d mut sdc::Mem<N>,
) -> Result<nrf_sdc::SoftdeviceController<'d>, nrf_sdc::Error> {
    sdc::Builder::new()?
        .support_adv()
        .support_peripheral()
        .support_dle_peripheral()
        .support_phy_update_peripheral()
        .support_le_2m_phy()
        .peripheral_count(1)?
        .buffer_cfg(L2CAP_MTU as u16, L2CAP_MTU as u16, L2CAP_TXQ, L2CAP_RXQ)?
        .build(p, rng, mpsl, mem)
}

pub fn ble_addr() -> [u8; 6] {
    let ficr = pac::FICR;
    let high = u64::from(ficr.deviceid(1).read());
    let addr = high << 32 | u64::from(ficr.deviceid(0).read());
    let addr = addr | 0x0000_c000_0000_0000;
    defmt::unwrap!(addr.to_le_bytes()[..6].try_into())
}
