use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use joydev::{Device, DeviceEvent, Error, GenericEvent};
use joydev::event_codes::AbsoluteAxis;
use joydev::event_codes::Key;

#[derive(Debug)]
struct JoyInfo {
    right_h: i16,
    right_v: i16,
    left_h: i16,
    left_v: i16,
    cross: bool,
    circle: bool,
    square: bool,
    triangle: bool,
    r1: bool,
    r2: bool,
    r3: bool,
    l1: bool,
    l2: bool,
    l3: bool,
    select: bool,
    start: bool,
}

fn main() -> Result<(), Error> {
    let running = Arc::new(AtomicBool::new(true));

    {
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");
    }

    let device = match Device::open("/dev/input/js0") {
        Ok(device) => device,
        Err(_) => panic!("No controller found at /dev/input/js0"),
    };

    let mut info = JoyInfo {
        right_h: 0,
        right_v: 0,
        left_h: 0,
        left_v: 0,
        cross: false,
        circle: false,
        square: false,
        triangle: false,
        r1: false,
        r2: false,
        r3: false,
        l1: false,
        l2: false,
        l3: false,
        select: false,
        start: false,
    };

    while running.load(Ordering::SeqCst) {
        'inner: loop {
            let event = match device.get_event() {
                Err(error) => match error {
                    Error::QueueEmpty => break 'inner,
                    _ => panic!("{}: {:?}", "called `Result::unwrap()` on an `Err` value", &error),
                },
                Ok(event) => event,
            };
            match event {
                DeviceEvent::Axis(ref event) => {
                    match event.axis() {
                        AbsoluteAxis::LeftX | AbsoluteAxis::Hat0X => info.left_h = GenericEvent::value(event),
                        AbsoluteAxis::LeftY | AbsoluteAxis::Hat0Y => info.left_v = GenericEvent::value(event),
                        AbsoluteAxis::LeftZ => info.right_h = GenericEvent::value(event),
                        AbsoluteAxis::RightZ => info.right_v = GenericEvent::value(event),
                        _ => panic!("Unknown axis used in event."),
                    };
                },
                DeviceEvent::Button(ref event) => {
                    let val = if GenericEvent::value(event) == 0 { false } else { true };
                    match event.button() {
                        Key::ButtonBase => info.l2 = val,
                        Key::ButtonBase2 => info.r2 = val,
                        Key::ButtonBase3 => info.select = val,
                        Key::ButtonBase4 => info.start = val,
                        Key::ButtonBase5 => info.l3 = val,
                        Key::ButtonBase6 => info.r3 = val,
                        Key::ButtonPinkie => info.r1 = val,
                        Key::ButtonThumb => info.circle = val,
                        Key::ButtonThumb2 => info.cross = val,
                        Key::ButtonTrigger => info.triangle = val,
                        Key::ButtonTop => info.square = val,
                        Key::ButtonTop2 => info.l1 = val,
                        _ => panic!("Unknown key pressed."),
                    };
                },
            }
            println!("{:?}", info);
        }
    }
    Ok(())
}
