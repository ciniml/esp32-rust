#![no_std]
#![feature(alloc)] 

use idf;
use idf::IdfError;

use freertos_rs::*;
use peripheral::*;
use embedded_hal::digital::v2::*;
use embedded_hal::blocking::delay::DelayMs;
use ili9341::*;

#[derive(Debug)]
pub enum M5StackError {
    Generic,
    IdfError(IdfError),
}

struct FreeRtosDelayMs {}
impl DelayMs<u16> for FreeRtosDelayMs {
    fn delay_ms(&mut self, ms: u16) {
        TaskDelay::new().delay_until(Duration::ms(ms as u32));
    }
}

type M5StackIli = Ili9341<SpiDeviceBusLock<()>, NormalGpioV1, NormalGpioV1, NormalGpioV1>;

pub fn new_lcd(bus: &mut SpiBus, pin_cs: GpioPin, pin_dc: GpioPin, pin_rst: GpioPin, pin_bl: GpioPin) -> Result<M5StackIli, M5StackError> {
    let spi_device_config = SpiDeviceInterfaceConfig {
        cs_pin: Some(pin_cs),
        clock_speed_hz: 10000000,
        ..Default::default()
    };
    let mut dc = pin_dc.normal();
    let mut rst = pin_rst.normal();
    let mut bl = pin_bl.normal();
    let mut cs = pin_cs.normal();
    let result = dc.configure(GpioConfig::output())
        .and_then(|_| rst.configure(GpioConfig::output()))
        .and_then(|_| cs.configure(GpioConfig::output()))
        .and_then(|_| bl.configure(GpioConfig::output()));
    if let Err(err) = result {
        return Err(M5StackError::IdfError(err))
    }
    dc.set_high();
    rst.set_low();
    bl.set_high();
    cs.set_high();

    let mut delay = FreeRtosDelayMs{};
    let device = bus.add_device(spi_device_config, |_| {}, |_| {});
    match device {
        Ok(device) => Ili9341::new(device, cs.to_v1(), dc.to_v1(), rst.to_v1(), &mut delay).map_err(|_| M5StackError::Generic),
        Err(err) => Err(M5StackError::IdfError(err)),
    }
}

