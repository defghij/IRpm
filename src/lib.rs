#![no_std]
#![feature(abi_avr_interrupt)]

pub mod types {
    use arduino_hal::port::mode::Output;
    use avr_hal_generic::{
        port::mode::Input,
        usart::{
            Usart,
            UsartWriter,
            UsartReader,
        },
        //clock::MHz16
    };
    use atmega_hal::{
        port::{
            PD1,
            PD0,
            //PD7
        }, Atmega
    };

    type Device = Atmega;
    type DeviceInterface = atmega_hal::pac::USART0;
    type InputPin = avr_hal_generic::port::Pin<Input, PD0>;
    type OutputPin = avr_hal_generic::port::Pin<Output, PD1>;
    type BaudRate = avr_hal_generic::clock::MHz16;
    pub type SerialConsole = Usart<Device, DeviceInterface, InputPin, OutputPin, BaudRate>;
    pub type SerialReader = UsartReader<Device, DeviceInterface, InputPin, OutputPin, BaudRate>;
    pub type SerialWriter = UsartWriter<Device, DeviceInterface, InputPin, OutputPin, BaudRate>;

    pub type IRLed = arduino_hal::port::Pin<arduino_hal::port::mode::Output, atmega_hal::port::PB5>;
    pub type StatusLed = arduino_hal::port::Pin<arduino_hal::port::mode::Output, atmega_hal::port::PB4>;

}

pub mod timers {
    use avr_device::atmega328p::TC0;
    static mut TIMER0_COUNTER: u32 = 0;
        
    ///////////////////////////////////////////////////////////////////////////////////
    //// Interrupt Zero: Time stamp
    const TM0_FREQUENCY: u32 = 1000;
    const TM0_PRESCALE:  u32 = 64;
    const TM0_CMR:       u32 = (16_000_000 / (TM0_PRESCALE * TM0_FREQUENCY)) - 1;

    ///  # TIMER
    ///  The below sets up a 1KHz, or 1,000 severy second, interrupt timer.
    ///  We use TIMER0 because it has a 8b compare which is acceptable because
    ///  the compare value is smaller than 2^8.  
    ///  ---------------------------------------------------------------------
    ///   interrupt_frequency = 1000 (Hz) 
    ///                       = 16_000_000 / (prescaler * cmp_match_reg + 1)
    ///   then,
    ///   cmp_match_reg = (16_000_000 / (prescaler * interrupt_frequency)) - 1
    ///                 = (16_000_000 / (64 * 1000) ) - 1
    ///                 = (16_000_000 / 64_000 ) - 1
    ///                 = 246              // < 2^8
    ///  ---------------------------------------------------------------------
    pub fn init(timer: TC0) {
        let timer0: TC0 = timer;                                     // Time with 8b compare
        timer0.tccr0a.write(| w: &mut _ | unsafe { w.bits(0) }    ); // Timer/Counter Control Register A 
        timer0.tccr0b.write(| w: &mut _ | w.cs0().prescale_64()   ); // Timer/Counter Control Register B: Clock Select
        timer0.ocr0a.write( | w: &mut _ | w.bits(TM0_CMR as u8)       ); // Output Compare Register
        timer0.tcnt0.write( | w: &mut _ | w.bits(0)               ); // Timer Counter
        timer0.timsk0.write(| w: &mut _ | w.ocie0a().set_bit()    ); // Enable timer interrupt
    }

    #[avr_device::interrupt(atmega328p)]
    fn TIMER0_COMPA() {
        avr_device::interrupt::free(|_cs| {
            unsafe {
                // Safety: This is the _only_ assignment to this memory location.
                TIMER0_COUNTER += 1;
            }
        });
    }

    #[inline(always)]
    pub fn millis() -> u32 {
        return unsafe { TIMER0_COUNTER };
    }
}
