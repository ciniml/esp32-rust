#![no_std]
#![feature(lang_items, alloc_error_handler, alloc)]
use core::panic::PanicInfo;
use core::alloc::{GlobalAlloc, Layout};
use core::fmt;
use core::fmt::Write;
use core::convert::From;
use core::convert::Into;
use core::ptr;
use core::mem;
use core::str;
use rand::prelude::*;

use m5stack::*;
use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line, Rect};
use embedded_graphics::image::Image16BPP;
use embedded_graphics::image::Image8BPP;

use freertos_rs::*;

extern crate alloc;
use alloc::rc::Rc;
use alloc::sync::Arc;
use alloc::vec::Vec;

use idf::*;
use embedded_hal::blocking::spi::Write as spiWrite; 

use peripheral::*;

extern "C" {
    fn malloc(size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
    fn write(file: isize, buffer: *const u8, count: usize) -> usize;
    fn lcd_print(buffer: *const u8, count: usize) -> usize;

    #[no_mangle]
    static esp_wifi_init_config_default: wifi_init_config_t;
}

struct LibcAllocator;
unsafe impl GlobalAlloc for LibcAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() > 8 {
            panic!("Unsupported alignment")
        }
        malloc(layout.size())
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr)
    }
}

struct Stdout;
impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            let buffer = s.as_bytes();
            if write(0, buffer.as_ptr(), buffer.len()) > 0 {
                Ok(())
            } else {
                Err(()).map_err(|_| fmt::Error)
            }
        }
    }
}

fn stdout() -> Stdout {
    Stdout
}

fn print_str(s: &str) -> Result<(), ()> {
    stdout().write_str(s).map_err(drop)
}
fn print_fmt(args: fmt::Arguments) -> Result<(), ()> {
    stdout().write_fmt(args).map_err(drop)
}



#[global_allocator]
static A: LibcAllocator = LibcAllocator;


unsafe extern "C" fn event_handler(ctx: *mut std::os::raw::c_void, event: *mut system_event_t) -> esp_err_t {
    match (*event).event_id {
        system_event_id_t_SYSTEM_EVENT_STA_START => {
            esp_wifi_connect();
        }
        system_event_id_t_SYSTEM_EVENT_STA_GOT_IP => {
            print_str("Got IP\n");
            let mut addrCStr = ip4addr_ntoa(&((*event).event_info.got_ip.as_ref().ip_info.ip));
            let mut addr = Vec::new();
            while *addrCStr != 0 {
                addr.push(*addrCStr as u8);
                addrCStr = addrCStr.offset(1);
            }
            if let Ok(s) = str::from_utf8(addr.as_ref()) {
                print_str(s);
            }
        }
        system_event_id_t_SYSTEM_EVENT_STA_DISCONNECTED => {
            print_str("Disconnected\n");
        }
        _ => {
            ;
        }
    }
    0
}

#[no_mangle]
pub extern fn rust_main() {
    print_str("Hello from Rust!\n");
    print_fmt(format_args!("Hello formatted from {}\n", "Rust"));

    
    
    let queue = Arc::new( Queue::<(i32, i32, i32, i32, u32)>::new(32).unwrap() );
    let queueDrawTask = queue.clone();
    let _drawTask = Task::new().name("line task").stack_size(4096).core(1).start(move || {
        let spi_bus_config = SpiBusConfig {
            mosi_pin: GpioPin23,
            miso_pin: GpioPin19,
            sclk_pin: GpioPin18,
            quadwp_pin: None,
            quadhd_pin: None,
            max_transfer_size: 1024,
        };
        let mut spi_bus = SpiBus::new(SpiHostDevice::Vspi, spi_bus_config, 0).unwrap();
        print_fmt(format_args!("Initializing LCD...\n"));
        let mut display = new_lcd(&mut spi_bus, GpioPin14, GpioPin27, GpioPin33, GpioPin32).unwrap();
        
        //display.draw(Rect::new(Coord::new(0, 0), Coord::new(320, 240)).fill(Some(0x0000u16)));
        display.draw(
            Font6x8::render_str("Hello World!")
                .stroke(Some(0xfffu16))
                .translate(Coord::new(5, 50)),
        );

        let image = Image8BPP::<u8>::new(include_bytes!("../../../fuga_8.raw"), 240, 240);
        display.draw(&image);

        loop {
            if let Ok((x0, y0, x1, y1, color)) = queueDrawTask.receive(Duration::infinite()) {
                //m5display.drawLine(x0, y0, x1, y1, color);
                //TaskDelay::new().delay_until(Duration::ms(10));
                display.draw(Line::new(Coord::new(x0, y0), Coord::new(x1, y1)).stroke(Some(color as u16)));
            }
            unsafe {
                esp_task_wdt_reset();
            }
        }
    }).unwrap();
    


    let mainTask = Task::current().unwrap();
    let queueRequestTask = queue.clone();
    let _requestTask = Task::new().name("rand task").stack_size(4096).core(0).start(move || {
        let seed:[u8; 16] = [7; 16];
        let mut rng = SmallRng::from_seed(seed);
        for i in 0..100000 {
            let x0:i32 = rng.gen_range(240, 320);
            let y0:i32 = rng.gen_range(0, 240);
            let x1:i32 = rng.gen_range(240, 320);
            let y1:i32 = rng.gen_range(0, 240);
            let color:u32 = rng.gen_range(0, 0x10000);

            queueRequestTask.send((x0, y0, x1, y1, color), Duration::infinite());
            unsafe {
                esp_task_wdt_reset();
            }
        }
        mainTask.notify(TaskNotification::SetBits(0x01));
    }).unwrap();
    
    unsafe {
        nvs_flash_init();
        tcpip_adapter_init();

        esp_event_loop_init(Some(event_handler), ptr::null_mut());

        let wifiInitConfig = esp_wifi_init_config_default;
        let mut wifiConfig = mem::zeroed::<wifi_config_t>();
        let ssid = "ssid\0";
        let password = "password\0";
        wifiConfig.sta.as_mut().ssid[..ssid.len()].clone_from_slice(ssid.as_bytes());
        wifiConfig.sta.as_mut().password[..password.len()].clone_from_slice(password.as_bytes());

        let result = esp_wifi_init(&wifiInitConfig).as_result()
            .and_then(|_| esp_wifi_set_mode(wifi_mode_t_WIFI_MODE_STA).as_result())
            .and_then(|_| esp_wifi_set_config(esp_interface_t_ESP_IF_WIFI_STA, &mut wifiConfig).as_result())
            .and_then(|_| esp_wifi_start().as_result());
        if let Err(code) = result {
            print_fmt(format_args!("Failed to initialize Wi-Fi - {}\n", code));
        }
    }
    
    let mainTask = Task::current().unwrap();
    mainTask.wait_for_notification(0x01, 0x01, Duration::infinite());
}   

#[lang = "panic_impl"]
#[no_mangle]
pub extern fn rust_begin_panic(_info: &PanicInfo) -> ! {
    print_fmt(format_args!("Panic: {:?}", _info));
    loop {}
}

#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
    print_fmt(format_args!("OOM: {:?}", _layout));
    loop {}
}
