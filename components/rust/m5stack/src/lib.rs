#![no_std]
#![feature(alloc)] 

use idf;
use idf::IdfError;

use freertos_rs::*;
use peripheral::*;
use embedded_hal::digital::v2::*;

#[derive(Debug)]
pub enum LcdError {
    Generic,
    IdfError(IdfError),
}

pub struct Lcd {
    spi: SpiDeviceRef<bool>,
    pin_dc: NormalGpio,
    pin_rst:NormalGpio,
    pin_bl: NormalGpio,
    pin_cs: NormalGpio,
}

const TFT_NOP:u8 = 0x00;
const TFT_SWRST:u8 = 0x01;

const TFT_CASET:u8 = 0x2A;
const TFT_PASET:u8 = 0x2B;
const TFT_RAMWR:u8 = 0x2C;

const TFT_RAMRD:u8 = 0x2E;
const TFT_IDXRD:u8 = 0xDD; // ILI9341 only, indexed control register read

const TFT_MADCTL:u8 = 0x36;
const TFT_MAD_MY:u8 = 0x80;
const TFT_MAD_MX:u8 = 0x40;
const TFT_MAD_MV:u8 = 0x20;
const TFT_MAD_ML:u8 = 0x10;
const TFT_MAD_BGR:u8 = 0x08;
const TFT_MAD_MH:u8 = 0x04;
const TFT_MAD_RGB:u8 = 0x00;

const TFT_INVOFF:u8 = 0x20;
const TFT_INVON:u8 = 0x21;

const ILI9341_NOP:u8 = 0x00;
const ILI9341_SWRESET:u8 = 0x01;
const ILI9341_RDDID:u8 = 0x04;
const ILI9341_RDDST:u8 = 0x09;

const ILI9341_SLPIN:u8 = 0x10;
const ILI9341_SLPOUT:u8 = 0x11;
const ILI9341_PTLON:u8 = 0x12;
const ILI9341_NORON:u8 = 0x13;

const ILI9341_RDMODE:u8 = 0x0A;
const ILI9341_RDMADCTL:u8 = 0x0B;
const ILI9341_RDPIXFMT:u8 = 0x0C;
const ILI9341_RDIMGFMT:u8 = 0x0A;
const ILI9341_RDSELFDIAG:u8 = 0x0F;

const ILI9341_INVOFF:u8 = 0x20;
const ILI9341_INVON:u8 = 0x21;
const ILI9341_GAMMASET:u8 = 0x26;
const ILI9341_DISPOFF:u8 = 0x28;
const ILI9341_DISPON:u8 = 0x29;

const ILI9341_CASET:u8 = 0x2A;
const ILI9341_PASET:u8 = 0x2B;
const ILI9341_RAMWR:u8 = 0x2C;
const ILI9341_RAMRD:u8 = 0x2E;

const ILI9341_PTLAR:u8 = 0x30;
const ILI9341_VSCRDEF:u8 = 0x33;
const ILI9341_MADCTL:u8 = 0x36;
const ILI9341_VSCRSADD:u8 = 0x37;
const ILI9341_PIXFMT:u8 = 0x3A;

const ILI9341_WRDISBV:u8 = 0x51;
const ILI9341_RDDISBV:u8 = 0x52;
const ILI9341_WRCTRLD:u8 = 0x53;

const ILI9341_FRMCTR1:u8 = 0xB1;
const ILI9341_FRMCTR2:u8 = 0xB2;
const ILI9341_FRMCTR3:u8 = 0xB3;
const ILI9341_INVCTR:u8 = 0xB4;
const ILI9341_DFUNCTR:u8 = 0xB6;

const ILI9341_PWCTR1:u8 = 0xC0;
const ILI9341_PWCTR2:u8 = 0xC1;
const ILI9341_PWCTR3:u8 = 0xC2;
const ILI9341_PWCTR4:u8 = 0xC3;
const ILI9341_PWCTR5:u8 = 0xC4;
const ILI9341_VMCTR1:u8 = 0xC5;
const ILI9341_VMCTR2:u8 = 0xC7;

const ILI9341_RDID4:u8 = 0xD3;
const ILI9341_RDINDEX:u8 = 0xD9;
const ILI9341_RDID1:u8 = 0xDA;
const ILI9341_RDID2:u8 = 0xDB;
const ILI9341_RDID3:u8 = 0xDC;
const ILI9341_RDIDX:u8 = 0xDD; // TBC

const ILI9341_GMCTRP1:u8 = 0xE0;
const ILI9341_GMCTRN1:u8 = 0xE1;

const ILI9341_MADCTL_MY:u8 = 0x80;
const ILI9341_MADCTL_MX:u8 = 0x40;
const ILI9341_MADCTL_MV:u8 = 0x20;
const ILI9341_MADCTL_ML:u8 = 0x10;
const ILI9341_MADCTL_RGB:u8 = 0x00;
const ILI9341_MADCTL_BGR:u8 = 0x08;
const ILI9341_MADCTL_MH:u8 = 0x04;


impl Lcd {
    pub fn new(bus: &mut SpiBus, pin_cs: GpioPin, pin_dc: GpioPin, pin_rst: GpioPin, pin_bl: GpioPin) -> Result<Lcd, LcdError> {
        let spi_device_config = SpiDeviceInterfaceConfig {
            cs_pin: None, //Some(pin_cs),
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
            return Err(LcdError::IdfError(err))
        }
        dc.set_high();
        rst.set_low();
        bl.set_low();
        cs.set_high();

        let device = bus.add_device(spi_device_config, |_|{}, |_|{}); /*move |dc:&bool| {
            let mut dc_pin = pin_dc.normal();
            dc_pin.set_level(*dc);
        }, |_| {
        });*/
        match device {
            Ok(device) => Ok(Lcd{spi: device, pin_dc: dc, pin_rst: rst, pin_bl: bl, pin_cs: cs}),
            Err(err) => Err(LcdError::IdfError(err)),
        }
    }

    pub fn reset(&mut self) -> Result<(), LcdError> {
        self.pin_rst.set_low();
        TaskDelay::new().delay_until(Duration::ms(150));
        self.pin_rst.set_high();
        TaskDelay::new().delay_until(Duration::ms(150));

        let result = self.write_cmd_data(0xef, &[0x03, 0x80, 0x02])
            .and_then(|_| { self.write_cmd_data(0xcf, &[0x00, 0xc1, 0x30]) })
            .and_then(|_| { self.write_cmd_data(0xed, &[0x64, 0x03, 0x12, 0x81]) })
            .and_then(|_| { self.write_cmd_data(0xe8, &[0x85, 0x00, 0x78]) })
            .and_then(|_| { self.write_cmd_data(0xcb, &[0x39, 0x2c, 0x00, 0x34, 0x02]) })
            .and_then(|_| { self.write_cmd_data(0xf7, &[0x20]) })
            .and_then(|_| { self.write_cmd_data(0xea, &[0x00, 0x00]) })
            .and_then(|_| { self.write_cmd_data(ILI9341_PWCTR1, &[0x23]) })
            .and_then(|_| { self.write_cmd_data(ILI9341_PWCTR2, &[0x10]) })
            .and_then(|_| { self.write_cmd_data(ILI9341_VMCTR1, &[0x3e, 0x28]) })
            .and_then(|_| { self.write_cmd_data(ILI9341_VMCTR2, &[0x86]) })
            .and_then(|_| { self.write_cmd_data(ILI9341_MADCTL, &[0xa8]) })
            .and_then(|_| { self.write_cmd_data(ILI9341_PIXFMT, &[0x55]) })
            .and_then(|_| { self.write_cmd_data(ILI9341_FRMCTR1, &[0x00, 0x13]) })
            .and_then(|_| { self.write_cmd_data(ILI9341_DFUNCTR, &[0x08, 0x82, 0x27]) })
            .and_then(|_| { self.write_cmd_data(0xf2, &[0x00]) })
            .and_then(|_| { self.write_cmd_data(ILI9341_GAMMASET, &[0x01]) })
            .and_then(|_| { self.write_cmd_data(ILI9341_GMCTRP1, &[0x0F, 0x31, 0x2B, 0x0C, 0x0E, 0x08, 0x4E, 0xF1, 0x37, 0x07, 0x10, 0x03, 0x0E, 0x09, 0x00]) })
            .and_then(|_| { self.write_cmd_data(ILI9341_GMCTRN1, &[0x00, 0x0E, 0x14, 0x03, 0x11, 0x07, 0x31, 0xC1, 0x48, 0x08, 0x0F, 0x0C, 0x31, 0x36, 0x0F]) })
            .and_then(|_| { self.write_cmd(ILI9341_SLPOUT) });
        if let Err(err) = result {
            return Err(err);
        }
        TaskDelay::new().delay_until(Duration::ms(120));
        let result = self.write_cmd(ILI9341_DISPON)
            .and_then(|_| { self.write_cmd_data(TFT_MADCTL, &[TFT_MAD_BGR]) });
        if let Err(err) = result {
            return Err(err);
        }
        self.pin_bl.set_level(true)
            .map_err(|err| { LcdError::IdfError(err) })
    }

    pub fn read_id(&mut self) -> Result<[u8;3], LcdError> {
        let mut buffer = [0, 0, 0]; 
        self.write_cmd(0x04)
            .and_then(|_| self.read_data(&mut buffer))
            .and(Ok(buffer))
    }

    pub fn write_cmd(&mut self, command: u8) -> Result<(), LcdError> {
        self.pin_cs.set_low();
        let buffer = [command];
        let transaction = SpiTransaction::new_write(&buffer, false);
        let mut device = self.spi.as_ref().lock().unwrap();
        self.pin_dc.set_low();
        match device.transfer(transaction) {
            Ok(_) => { self.pin_cs.set_high(); Ok(()) },
            Err(err) => { self.pin_cs.set_high(); Err(LcdError::IdfError(err)) },
        }
    }
    
    pub fn write_data(&mut self, data: &[u8]) -> Result<(), LcdError> {
        self.pin_cs.set_low();
        let transaction = SpiTransaction::new_write(data, true);
        let mut device = self.spi.as_ref().lock().unwrap();
        self.pin_dc.set_high();
        match device.transfer(transaction) {
            Ok(_) => { self.pin_cs.set_high(); Ok(()) },
            Err(err) => { self.pin_cs.set_high(); Err(LcdError::IdfError(err)) },
        }
    }

    pub fn write_cmd_data(&mut self, command: u8, values: &[u8]) -> Result<(), LcdError> {
        self.write_cmd(command)
            .and_then(|_| self.write_data(values))
    }

    pub fn read_data(&mut self, data: &mut [u8]) -> Result<(), LcdError> {
        let transaction = SpiTransaction::new_read(data, true);
        let mut device = self.spi.as_ref().lock().unwrap();
        self.pin_dc.set_high();
        match device.transfer(transaction) {
            Ok(_) => Ok(()),
            Err(err) => Err(LcdError::IdfError(err)),
        }
    }
    

}
