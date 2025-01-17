/*!
This module provides target specific integration with the [`mcan`] crate.

MCAN is an of-the-shelf peripheral that is synthesized and integrated
into, among the others, (some) x7x MCUs.

Instance of the  [`Dependencies`] struct is necessary in order to make `mcan`
abstractions operational.

More information regarding the `mcan` API can be found in its
documentation.

[`mcan`]: https://crates.io/crates/mcan
*/

use fugit::HertzU32;

use crate::{
    clocks::{Clock, HostClock, Pck, Pck5, PeripheralIdentifier},
    generics,
    pio::*,
};

/// Metadata for a CAN peripheral
pub trait CanMeta: generics::Sealed + mcan_core::CanId {
    /// Constant mapping from the CAN peripheral instance to the matching
    /// [`PeripheralIdentifier`]
    const PID: PeripheralIdentifier;
    /// Corresponding low-level register PAC type
    type REG;

    /// Method providing the pointer to the beginning of the eligible message
    /// RAM
    ///
    /// More details at [`mcan_core::Dependencies::eligible_message_ram_start`]
    fn eligible_message_ram_start(matrix: &crate::pac::MATRIX) -> usize;
}

/// Identity type for `MCAN0`
pub enum Can0 {}

impl CanMeta for Can0 {
    const PID: PeripheralIdentifier = PeripheralIdentifier::MCAN0;
    type REG = crate::pac::MCAN0;
    fn eligible_message_ram_start(matrix: &crate::pac::MATRIX) -> usize {
        (matrix.ccfg_can0.read().can0dmaba().bits() as usize) << 16
    }
}

impl generics::Sealed for Can0 {}

unsafe impl mcan_core::CanId for Can0 {
    const ADDRESS: *const () = <Self as CanMeta>::REG::PTR as _;
}

/// Identity type for `MCAN1`
#[cfg(any(feature = "pins-100", feature = "pins-144"))]
pub enum Can1 {}

#[cfg(any(feature = "pins-100", feature = "pins-144"))]
impl CanMeta for Can1 {
    const PID: PeripheralIdentifier = PeripheralIdentifier::MCAN1;
    type REG = crate::pac::MCAN1;
    fn eligible_message_ram_start(matrix: &crate::pac::MATRIX) -> usize {
        (matrix.ccfg_sysio.read().can1dmaba().bits() as usize) << 16
    }
}

#[cfg(any(feature = "pins-100", feature = "pins-144"))]
impl generics::Sealed for Can1 {}

#[cfg(any(feature = "pins-100", feature = "pins-144"))]
unsafe impl mcan_core::CanId for Can1 {
    const ADDRESS: *const () = <Self as CanMeta>::REG::PTR as _;
}

/// Struct enclosing all the dependencies required to bootstrap `Id` instance of
/// MCAN.
///
/// Its construction means that all platform-specific prerequisites are in place
/// and `mcan` abstractions are operational.
pub struct Dependencies<Id: CanMeta, Tx, Rx> {
    #[allow(dead_code)]
    reg: Id::REG,
    eligible_message_ram_start: usize,
    #[allow(dead_code)]
    pins: (Tx, Rx),
    host_clock_freq: HertzU32,
    can_clock_freq: HertzU32,
}

impl<Id: CanMeta, Tx, Rx> Dependencies<Id, Tx, Rx> {
    /// Create an instance of `Dependencies` struct.
    ///
    /// This struct implements [`mcan_core::Dependencies`] trait, making it
    /// possible to construct an instance of `mcan::bus::CanConfigurable`
    ///
    /// # Safety
    /// While [`Dependencies`] instance exists:
    /// - matrix.ccfg_can0.can0dmaba value cannot change
    /// - matrix.ccfg_sysio.read().can1dmaba() value cannot change
    pub unsafe fn new(
        reg: Id::REG,
        matrix: &crate::pac::MATRIX,
        pins: (Tx, Rx),
        pck: &Pck<Pck5>,
        hclk: &mut HostClock,
    ) -> Self {
        hclk.enable_peripheral(Id::PID);

        let host_clock_freq = hclk.freq();
        let can_clock_freq = pck.freq();

        // CAN high precision clock frequency must be slower or equal to the CPU clock
        assert!(can_clock_freq <= host_clock_freq);

        Self {
            reg,
            eligible_message_ram_start: Id::eligible_message_ram_start(matrix),
            pins,
            host_clock_freq,
            can_clock_freq,
        }
    }
}

unsafe impl<Id, Tx, Rx> mcan_core::Dependencies<Id> for Dependencies<Id, Tx, Rx>
where
    Id: CanMeta,
    Rx: RxPin<ValidFor = Id>,
    Tx: TxPin<ValidFor = Id>,
{
    fn host_clock(&self) -> fugit::HertzU32 {
        self.host_clock_freq
    }

    fn can_clock(&self) -> fugit::HertzU32 {
        self.can_clock_freq
    }

    fn eligible_message_ram_start(&self) -> *const () {
        self.eligible_message_ram_start as _
    }
}

trait RxPin {
    type ValidFor: mcan_core::CanId;
}

trait TxPin {
    type ValidFor: mcan_core::CanId;
}

impl RxPin for Pin<PB3, PeripheralA> {
    type ValidFor = Can0;
}

impl TxPin for Pin<PB2, PeripheralA> {
    type ValidFor = Can0;
}

#[cfg(feature = "pins-144")]
impl RxPin for Pin<PC12, PeripheralC> {
    type ValidFor = Can1;
}

#[cfg(feature = "pins-100")]
impl RxPin for Pin<PD28, PeripheralB> {
    type ValidFor = Can1;
}

#[cfg(feature = "pins-144")]
impl TxPin for Pin<PC14, PeripheralC> {
    type ValidFor = Can1;
}

#[cfg(any(feature = "pins-100", feature = "pins-144"))]
impl TxPin for Pin<PD12, PeripheralB> {
    type ValidFor = Can1;
}
