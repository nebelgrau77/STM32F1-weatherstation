//! simple weather station using a Bosch BME280 sensor
//! and an SSD1306 OLED display
//! 
//! in order to share the I2C bus between the two devices
//! the shared-bus crate is used
//! 
//! this project uses the STM32F103C8T6 "blue pill" board
//! 
//! the BME280 driver requires BlockingI2c trait
//! available in the the STM32F1xx-HAL crate but not in STM32F0xx nor STM32F4xx
//! 
//! as the BME280 initialization consumes the delay instance
//! the delay in the loop is done blocking the program for n instructions


#![no_std]
#![no_main]

extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate stm32f1xx_hal as hal;
extern crate shared_bus;
extern crate cortex_m;

use bme280::BME280;

use cortex_m_rt::entry;

use hal::{
    i2c::{BlockingI2c, DutyCycle, Mode},
    prelude::*,
    stm32,
    delay::Delay,
    
};

use ssd1306::{prelude::*, Builder as SSD1306Builder};

use core::fmt;
use core::fmt::Write;
use arrayvec::ArrayString;


#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);
    
    
    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 400_000.hz(),
            duty_cycle: DutyCycle::Ratio2to1,
        },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );

    // shared-bus manager created
    let manager = shared_bus::CortexMBusManager::new(i2c);

    // delay provider for the BME280
    let delay = Delay::new(cp.SYST, clocks);
    
    // BME280 sensor initiation
    let bme280_i2c_addr = 0x76;
    let mut bme280 = BME280::new(manager.acquire(), bme280_i2c_addr, delay);    
    bme280.init().unwrap();
    
    //ssd1306 i2c address: not required in this case
    //let ssd1306_i2c_addr = 0x3c;

    // display initiated in TerminalMode
    let mut disp: TerminalMode<_> = SSD1306Builder::new().size(DisplaySize::Display128x32).connect_i2c(manager.acquire()).into();
        
    disp.init().unwrap();
    disp.clear().unwrap();
    
    loop {
        
        // create an empty buffer for the display
        let mut output = ArrayString::<[u8; 64]>::new();
        
        // get the sensor measurements
        let measurements = bme280.measure().unwrap();
                
        // format and send the buffer to the display
        format(&mut output, measurements.temperature as u8, measurements.humidity as u8);           
        disp.write_str(output.as_str()).unwrap();

        // just wait for many cycles
        cortex_m::asm::delay(1 * 84_000_000);

    }

}

// helper function for the display data formatting
// the buffer must always contain 64 characters to fill the whole 128x32 space
// otherwise with every refresh the contents will be moved across the screen

fn format(buf: &mut ArrayString<[u8; 64]>, temp: u8, hum: u8) {
    fmt::write(buf, format_args!("T:   {:02}C                        H:   {:02}%                        ", 
    temp, hum)).unwrap();
}
