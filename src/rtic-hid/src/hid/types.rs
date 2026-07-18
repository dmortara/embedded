#![deny(unsafe_code)]
use usbd_hid::descriptor::generator_prelude::*;

// Standard USB HID Joystick — Generic Desktop page (0x01), Joystick usage (0x04).
// Ten u16 axes; logical range 0–65535 is derived from the type by the macro.
// ADC values (12-bit, 0–4095) are shifted left 4 to fill the full u16 range.
//
// Each axis gets its own USAGE local item so the HID host assigns the correct
// usage regardless of REPORT_COUNT (local items are consumed per main item).
// Axis usages: 0x30 X · 0x31 Y · 0x32 Z · 0x33 Rx · 0x34 Ry · 0x35 Rz
//              0x36 Slider · 0x37 Dial · 0x38 Wheel · 0x36 Slider (2nd)
//(usage = 0x30,) = { x=input;      };
//(usage = 0x31,) = { y=input;      };
#[gen_hid_descriptor(
    //(collection = 0x01, usage_page = 0x01, usage = 0x04) = {
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = JOYSTICK) = {
        (collection = LOGICAL,) = {
            (usage = X,) = { #[item_settings(data, variable, absolute)] x=input; };
            (usage = Y,) = { #[item_settings(data, variable, absolute)] y=input;      };
            (usage = 0x35,) = { #[item_settings(data, variable, absolute)] z=input;      };
            (usage = 0x33,) = { rx=input;     };
            (usage = 0x34,) = { ry=input;     };
            (usage = 0x35,) = { rz=input;     };
            (usage = 0x36,) = { slider=input; };
            (usage = 0x37,) = { dial=input;   };
            (usage = 0x38,) = { wheel=input;  };
            (usage = 0x36,) = { slider2=input;};
            (usage_min = 1, usage_max = 8 , usage_page = BUTTON ) = {
                #[packed_bits = 8] buttons= input;
            }
        };
    }
)]
pub struct JoystickReport {
    pub x: u16,
    pub y: u16,
    pub z: u16,
    pub rx: u16,
    pub ry: u16,
    pub rz: u16,
    pub slider: u16,
    pub dial: u16,
    pub wheel: u16,
    pub slider2: u16,
    pub buttons: u8,
}
