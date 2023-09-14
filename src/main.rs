/*

    Rotary Encoder

    Cool stuff:
        Rotary encoder algorithm
        millis implementation

*/

#![feature(abi_avr_interrupt)]
#![no_std]
#![no_main]

// MILLIS START
use core::cell;

use panic_halt as _;

const PRESCALER: u32 = 1024;
const TIMER_COUNTS: u32 = 125;

const MILLIS_INCREMENT: u32 = PRESCALER * TIMER_COUNTS / 16000;

static MILLIS_COUNTER: avr_device::interrupt::Mutex<cell::Cell<u32>> =
    avr_device::interrupt::Mutex::new(cell::Cell::new(0));

fn millis_init(tc0: arduino_hal::pac::TC0) {
    // Configure the timer for the above interval (in CTC mode)
    // and enable its interrupt.
    tc0.tccr0a.write(|w| w.wgm0().ctc());
    tc0.ocr0a.write(|w| w.bits(TIMER_COUNTS as u8));
    tc0.tccr0b.write(|w| match PRESCALER {
        8 => w.cs0().prescale_8(),
        64 => w.cs0().prescale_64(),
        256 => w.cs0().prescale_256(),
        1024 => w.cs0().prescale_1024(),
        _ => panic!(),
    });
    tc0.timsk0.write(|w| w.ocie0a().set_bit());

    // Reset the global millisecond counter
    avr_device::interrupt::free(|cs| {
        MILLIS_COUNTER.borrow(cs).set(0);
    });
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    avr_device::interrupt::free(|cs| {
        let counter_cell = MILLIS_COUNTER.borrow(cs);
        let counter = counter_cell.get();
        counter_cell.set(counter + MILLIS_INCREMENT);
    })
}

fn millis() -> u32 {
    avr_device::interrupt::free(|cs| MILLIS_COUNTER.borrow(cs).get())
}

/*ADD this to main

    millis_init(dp.TC0);

    // Enable interrupts globally
    unsafe { avr_device::interrupt::enable() };

*/

// MILLIS END

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    millis_init(dp.TC0);

    // Enable interrupts globally
    unsafe { avr_device::interrupt::enable() };

    ufmt::uwriteln!(&mut serial, "Hello from Arduino!\r").unwrap();

    let button = pins.d2.into_pull_up_input();
    let s1 = pins.d3.into_floating_input();
    let s2 = pins.d4.into_floating_input();

    let mut counter = 0i32;

    let mut ps1 = true;
    let mut ps2 = true;
    let mut pps1 = true;
    let mut pps2 = true;

    loop {
        let res_b = button.is_high();
        let res_s1 = s1.is_high();
        let res_s2 = s2.is_high();

        if (res_s1 == true && res_s2 == false)
            && (ps1 == false && ps2 == false)
            && (pps1 == false && pps2 == true)
        {
            counter -= 1;
        } else if (res_s1 == false && res_s2 == true)
            && (ps1 == false && ps2 == false)
            && (pps1 == true && pps2 == false)
        {
            counter += 1;
        }
        if !res_b {
            counter = 0;
        }

        let time = millis();

        if time % 10 == 0 {
            ufmt::uwrite!(
                &mut serial,
                "                            \rcounter: {}\r",
                counter
            )
            .unwrap();
        }
        // ufmt::uwriteln!(&mut serial, "res_s1: {}\t res_s2: {}\r", res_s1, res_s2).unwrap();
        if res_s1 != ps1 || res_s2 != ps2 {
            pps1 = ps1;
            pps2 = ps2;
            ps1 = res_s1;
            ps2 = res_s2;
        }
        // arduino_hal::delay_ms(5);
    }
}
