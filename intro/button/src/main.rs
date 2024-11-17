#![no_std]
#![no_main]

use core::cell::RefCell;

use critical_section::*;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{GpioPin, Input, Io, Level, Output, Pull},
    peripherals::Peripherals,
    prelude::*
};
use esp_println::println;

static BUTTON: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<Output>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    println!("Hello world!");

    // Set GPIO7 as an output, and set its state high initially.
    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // XXX: Don't forget to register a interrupt handler
    // TODO: How does the GPIO interrupt actually work?
    // XXX: How does interrupt work on esp32c3?
    io.set_interrupt_handler(handler);

    let led = Output::new(io.pins.gpio7, Level::Low);

    let mut button = Input::new(io.pins.gpio9, Pull::Up);

    // XXX: What's the relationship b/w pull-up and falling edge? :-)

    // let delay = Delay::new();

    critical_section::with(|cs| {
        button.listen(esp_hal::gpio::Event::FallingEdge);
        BUTTON
            // XXX: This is how you use critical_section::Mutex
            .borrow_ref_mut(cs)
            // XXX: This is the actual behavior you want: Set the value of
            // BUTTON (by relacing `None` with `button`).
            .replace(button);

        LED.borrow_ref_mut(cs).replace(led);
    });

    // Check the button state and set the LED state accordingly.
    loop {
        // delay.delay_millis(500);
        // led.toggle();

        // if button.is_high() {
        //     led.set_low();
        // } else {
        //     led.set_high();
        // }
    }
}

#[handler]
fn handler() {
    critical_section::with(|cs| {
        println!("GPIO interrupt");

        // XXX: Is this optimal?
        LED.borrow_ref_mut(cs).as_mut().unwrap().toggle();

        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();
    });
}