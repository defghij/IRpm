#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]


use panic_halt as _;
use avr_hal_generic::prelude::_unwrap_infallible_UnwrapInfallible;

use core::{sync::atomic::AtomicBool, mem::MaybeUninit};
use core::sync::atomic::Ordering;


use i_rpm::{
    types::{
        StatusLed,
        SerialConsole
    },
    timers
};

pub enum Edge {
    Falling = 0x02,
    Rising = 0x03,
}


static STATUS: AtomicBool = AtomicBool::new(false);
static mut RPM_COUNT:  u32 = 0;
static mut RPM:        u32 = 0;
static mut TIMEOLD:    u32 = 0;
static mut STATUS_LED: MaybeUninit<StatusLed> = MaybeUninit::uninit();

#[avr_device::interrupt(atmega328p)]
fn INT0() {
    unsafe { 
        RPM_COUNT += 1;
        if !cfg!(debug_assertions) { // then toggle status LED
            // Get a pointer to the sensor.
            let status_led: &mut StatusLed = &mut *STATUS_LED.as_mut_ptr();

            // Toggles status LED
            if STATUS.load(Ordering::Acquire) == false {
                STATUS.store(true, Ordering::Release);
                status_led.set_high();
            } else {
                STATUS.store(false, Ordering::Release);
                status_led.set_low();
            }   
        }
    }
}


#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    
    let mut ir_led = pins.d13.into_output(); // Turn on IR LED.
    ir_led.set_high();

    //let mut status_led = pins.d12.into_output(); // Use status_pin to flash along with interrupts.
    let mut serial: SerialConsole  = arduino_hal::default_serial!(dp, pins, 57200);


    
    dp.EXINT.eicra.modify(|_,w| { w.isc0().bits(Edge::Falling as u8) });// Configure INT0 for falling edge
    dp.EXINT.eimsk.modify(|_,w| { w.int0().set_bit() });                // Enable INT0 interrupt source

    STATUS.store(false, Ordering::Release);
    timers::init(dp.TC0);
    
    ufmt::uwrite!(serial, "millis,RPM\n").unwrap_infallible();

    unsafe { // BEGIN!
        STATUS_LED = MaybeUninit::new(pins.d12.into_output());
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        avr_device::interrupt::enable();
    }

    loop {
        arduino_hal::delay_ms(1000);
        avr_device::interrupt::free(|_cs| { 
            unsafe {
                RPM = 60 * 1000  / (timers::millis() - TIMEOLD) * RPM_COUNT;
                TIMEOLD = timers::millis();
                RPM_COUNT = 0;
            };
        
        });
        unsafe { ufmt::uwrite!(serial, "{},{}\n", timers::millis(), RPM).unwrap_infallible(); }
    }
}
