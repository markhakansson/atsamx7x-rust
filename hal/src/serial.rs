use crate::{ehal, nb};

// Smaller part have 3x UART & 2x USART
use crate::target_device::{UART0, UART1, UART2, USART0, USART1};
#[cfg(all(
    any(feature = "e70", feature = "s70"),
    any(feature = "pins-100", feature = "pins-144")
))]
use crate::target_device::{UART3, USART2};

#[cfg(all(
    any(feature = "e70", feature = "s70"),
    any(feature = "pins-100", feature = "pins-144")
))]
use crate::target_device::UART4;

use crate::target_device::{
    uart0::RegisterBlock as UARTRegisterBlock, usart0::RegisterBlock as USARTRegisterBlock,
};

pub struct Serial<P> {
    peripheral: P,
}

pub type Serial0 = Serial<UART0>;
pub type Serial1 = Serial<UART1>;
pub type Serial2 = Serial<UART2>;

#[cfg(all(
    any(feature = "e70", feature = "s70"),
    any(feature = "pins-100", feature = "pins-144")
))]
pub type Serial3 = Serial<UART3>;

#[cfg(all(
    any(feature = "e70", feature = "s70"),
    any(feature = "pins-100", feature = "pins-144")
))]
pub type Serial4 = Serial<UART4>;

pub type Serial5 = Serial<USART0>;
pub type Serial6 = Serial<USART1>;

#[cfg(all(
    any(feature = "e70", feature = "s70"),
    any(feature = "pins-100", feature = "pins-144")
))]
pub type Serial7 = Serial<USART2>;

//#[cfg(any(
//    feature = "?"))]
//pub type Serial8 = Serial<USART3>;

#[derive(Debug)]
pub enum Error {
    /// Buffer overrun
    Overrun,
    // omitted: other error variants
}

impl ehal::serial::Write<u8> for Serial<UART0> {
    type Error = Error;

    fn write(&mut self, word: u8) -> nb::Result<(), Error> {
        write_uart(&*self.peripheral, word)
    }

    fn flush(&mut self) -> nb::Result<(), Error> {
        flush_uart(&*self.peripheral)
    }
}

impl ehal::serial::Write<u8> for Serial<USART1> {
    type Error = Error;

    fn write(&mut self, word: u8) -> nb::Result<(), Error> {
        write_usart(&*self.peripheral, word)
    }

    fn flush(&mut self) -> nb::Result<(), Error> {
        flush_usart(&*self.peripheral)
    }
}

#[cfg(all(any(feature = "e70", feature = "s70"), feature = "pins-100"))]
impl ehal::serial::Write<u8> for Serial<UART3> {
    type Error = Error;

    fn write(&mut self, word: u8) -> nb::Result<(), Error> {
        write_uart(&*self.peripheral, word)
    }

    fn flush(&mut self) -> nb::Result<(), Error> {
        flush_uart(&*self.peripheral)
    }
}

fn write_uart(regs: &UARTRegisterBlock, word: u8) -> nb::Result<(), Error> {
    if regs.uart_sr.read().txempty().bit_is_clear() {
        Err(nb::Error::WouldBlock)
    } else {
        regs.uart_thr.write(|w| unsafe { w.txchr().bits(word) });
        Ok(())
    }
}

fn write_usart(regs: &USARTRegisterBlock, word: u8) -> nb::Result<(), Error> {
    if regs.us_csr_usart_mode().read().txempty().bit_is_clear() {
        Err(nb::Error::WouldBlock)
    } else {
        regs.us_thr
            .write(|w| unsafe { w.txchr().bits(word as u16) });
        Ok(())
    }
}

fn flush_uart(regs: &UARTRegisterBlock) -> nb::Result<(), Error> {
    if regs.uart_sr.read().txempty().bit_is_clear() {
        Err(nb::Error::WouldBlock)
    } else {
        Ok(())
    }
}

fn flush_usart(regs: &USARTRegisterBlock) -> nb::Result<(), Error> {
    if regs.us_csr_usart_mode().read().txempty().bit_is_clear() {
        Err(nb::Error::WouldBlock)
    } else {
        Ok(())
    }
}
