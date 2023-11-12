use log::*;
use std::{thread, time::Duration, sync::Arc, sync::Mutex};
use esp_idf_hal::{gpio::*, spi, delay::FreeRtos};
use ssd1331::{DisplayRotation, Ssd1331};
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, ascii::FONT_5X8, ascii::FONT_6X12, MonoTextStyle},
    image::Image,
    pixelcolor::{Rgb565},
    text::{Text},
    geometry::Point,
    primitives::{
        Circle, Line, Triangle, PrimitiveStyle,
    },
    prelude::*,
};
use tinybmp::Bmp;

pub enum LoggingStatus {
    Start,
    Stop,
}

pub enum WifiStatus {
    Connected,
    Disconnected,
}

type SPI<'d> = esp_idf_hal::spi::SpiDeviceDriver<'static, spi::SpiDriver<'static>>;
type DC<'d> = esp_idf_hal::gpio::PinDriver<'static, Gpio10, esp_idf_hal::gpio::Output>;
type RST<'d> = esp_idf_hal::gpio::PinDriver<'static, Gpio1, esp_idf_hal::gpio::Output>;

struct DisplayText {
    voltage: f32,
    current: f32,
    power: f32,
    interval: u32,
    message: String,
    battery: f32,
    status: LoggingStatus,
    wifi: WifiStatus,
    buffer_water_mark: u32,
}

pub struct DisplayPanel {
    txt: Arc<Mutex<DisplayText>>
}

impl DisplayPanel {

    pub fn new() -> DisplayPanel {
        DisplayPanel { txt: Arc::new(Mutex::new(
            DisplayText {voltage: 0.0,
                         message: "".to_string(),
                         current: 0.0,
                         power: 0.0,
                         interval: 0,
                         battery: 0.0,
                         status: LoggingStatus::Stop,
                         wifi: WifiStatus::Disconnected,
                         buffer_water_mark: 0,
                     })) }
    }

    pub fn start(&mut self,
        spi : SPI, dc: DC, mut rst : RST)
    {
        let txt = self.txt.clone();
        let _th = thread::spawn(move || {
            info!("Start Display Thread.");
            let mut delay = FreeRtos;
            let mut display = Ssd1331::new(spi, dc, DisplayRotation::Rotate180);
            let _ = display.reset(&mut rst, &mut delay);
            let _ = display.init();
            display.clear();
            let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
            let middle_style_white = MonoTextStyle::new(&FONT_6X12, Rgb565::WHITE);
            let middle_style_red = MonoTextStyle::new(&FONT_6X12, Rgb565::RED);
            let small_style_white = MonoTextStyle::new(&FONT_5X8, Rgb565::WHITE);
            let wifibmp = Bmp::from_slice(include_bytes!("./img/wifirev.bmp")).unwrap();
            let wifi_img: Image<Bmp<Rgb565>> = Image::new(&wifibmp, Point::new(76,1));
            let fill = PrimitiveStyle::with_fill(Rgb565::YELLOW);

            // Battery BMP
            let bat_x = 81;
            let bat_y = 33;
            let bat0 = Bmp::from_slice(include_bytes!("./img/battery-0.bmp")).unwrap();
            let bat0_img: Image<Bmp<Rgb565>> = Image::new(&bat0, Point::new(bat_x, bat_y));
            let bat20 = Bmp::from_slice(include_bytes!("./img/battery-20.bmp")).unwrap();
            let bat20_img: Image<Bmp<Rgb565>> = Image::new(&bat20, Point::new(bat_x, bat_y));
            let bat40 = Bmp::from_slice(include_bytes!("./img/battery-40.bmp")).unwrap();
            let bat40_img: Image<Bmp<Rgb565>> = Image::new(&bat40, Point::new(bat_x, bat_y));
            let bat60 = Bmp::from_slice(include_bytes!("./img/battery-60.bmp")).unwrap();
            let bat60_img: Image<Bmp<Rgb565>> = Image::new(&bat60, Point::new(bat_x, bat_y));
            let bat80 = Bmp::from_slice(include_bytes!("./img/battery-80.bmp")).unwrap();
            let bat80_img: Image<Bmp<Rgb565>> = Image::new(&bat80, Point::new(bat_x, bat_y));
            let bat100 = Bmp::from_slice(include_bytes!("./img/battery-100.bmp")).unwrap();
            let bat100_img: Image<Bmp<Rgb565>> = Image::new(&bat100, Point::new(bat_x, bat_y));
            let usbpwr = Bmp::from_slice(include_bytes!("./img/usb-power.bmp")).unwrap();
            let usbpwr_img: Image<Bmp<Rgb565>> = Image::new(&usbpwr, Point::new(bat_x, bat_y));

            // Number BMP
            let n0 = Bmp::from_slice(include_bytes!("./img/n0.bmp")).unwrap();
            let n0_img: Image<Bmp<Rgb565>> = Image::new(&n0, Point::zero());
            let n1 = Bmp::from_slice(include_bytes!("./img/n1.bmp")).unwrap();
            let n1_img: Image<Bmp<Rgb565>> = Image::new(&n1, Point::zero());
            let n2 = Bmp::from_slice(include_bytes!("./img/n2.bmp")).unwrap();
            let n2_img: Image<Bmp<Rgb565>> = Image::new(&n2, Point::zero());
            let n3 = Bmp::from_slice(include_bytes!("./img/n3.bmp")).unwrap();
            let n3_img: Image<Bmp<Rgb565>> = Image::new(&n3, Point::zero());
            let n4 = Bmp::from_slice(include_bytes!("./img/n4.bmp")).unwrap();
            let n4_img: Image<Bmp<Rgb565>> = Image::new(&n4, Point::zero());
            let n5 = Bmp::from_slice(include_bytes!("./img/n5.bmp")).unwrap();
            let n5_img: Image<Bmp<Rgb565>> = Image::new(&n5, Point::zero());
            let n6 = Bmp::from_slice(include_bytes!("./img/n6.bmp")).unwrap();
            let n6_img: Image<Bmp<Rgb565>> = Image::new(&n6, Point::zero());
            let n7 = Bmp::from_slice(include_bytes!("./img/n7.bmp")).unwrap();
            let n7_img: Image<Bmp<Rgb565>> = Image::new(&n7, Point::zero());
            let n8 = Bmp::from_slice(include_bytes!("./img/n8.bmp")).unwrap();
            let n8_img: Image<Bmp<Rgb565>> = Image::new(&n8, Point::zero());
            let n9 = Bmp::from_slice(include_bytes!("./img/n9.bmp")).unwrap();
            let n9_img: Image<Bmp<Rgb565>> = Image::new(&n9, Point::zero());
            let vv = Bmp::from_slice(include_bytes!("./img/v.bmp")).unwrap();
            let vv_img: Image<Bmp<Rgb565>> = Image::new(&vv, Point::new(68, 0));
            let dot = Bmp::from_slice(include_bytes!("./img/dot.bmp")).unwrap();
            let dot_img: Image<Bmp<Rgb565>> = Image::new(&dot, Point::zero());
            let minus = Bmp::from_slice(include_bytes!("./img/minus.bmp")).unwrap();
            let minus_img: Image<Bmp<Rgb565>> = Image::new(&minus, Point::zero());
            let mut digit_img = n0_img.translate(Point::new(0,0));

            let mut loopcount = 0;
            let mut battery_level = 0;
            let mut battery_queue : [f32;10] = [0.0;10];
            loop {
                let lck = txt.lock().unwrap();
                display.clear();
                let mut temp = lck.voltage;
                if temp != -999.0 {
                    dot_img.draw(&mut display).unwrap();                
                    vv_img.draw(&mut display).unwrap();
                    let mut digit_10 = 10.0;
                    let mut first_digit = true;
                    let mut pos_x = 0;
                    for digit in 0..=3 {
                        if pos_x >= 68 {
                            continue;
                        }
                        let num = (temp / digit_10) as i32;
                        if temp < 0.0 && digit == 0 {
                            digit_img = minus_img.translate(Point::new(pos_x, 0));
                            digit_img.draw(&mut display).unwrap();
                            pos_x += 20;
                        }
                        match num {
                            0 => {
                                if !first_digit || digit > 0 {
                                    digit_img = n0_img.translate(Point::new(pos_x, 0));
                                    pos_x += 20;
                                }
                            },
                            1 | -1 => {
                                digit_img = n1_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                                pos_x += 20;
                            },
                            2 | -2 => {
                                digit_img = n2_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                                pos_x += 20;
                            },
                            3 | -3 => {
                                digit_img = n3_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                                pos_x += 20;
                            },
                            4 | -4 => {
                                digit_img = n4_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                                pos_x += 20;
                            },
                            5 | -5 => {
                                digit_img = n5_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                                pos_x += 20;
                            },
                            6 | -6 => {
                                digit_img = n6_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                                pos_x += 20;
                            },
                            7 | -7 => {
                                digit_img = n7_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                                pos_x += 20;
                            },
                            8 | -8 => {
                                digit_img = n8_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                                pos_x += 20;
                            },
                            9 | -9 => {
                                digit_img = n9_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                                pos_x += 20;
                            },
                            _ => {}
                        }
                        digit_img.draw(&mut display).unwrap();
                        if digit == 1 {
                            digit_img = dot_img.translate(Point::new(pos_x, 0));
                            digit_img.draw(&mut display).unwrap();
                            pos_x += 8;
                        }
                        temp = temp - digit_10 * (num as f32);
                        digit_10 /= 10.0;
                    }
                }
                else {
                    Text::new(&format!("{}", lck.message), Point::new(1, 20), style).draw(&mut display).unwrap();
                }

                match lck.status {
                    LoggingStatus::Start => {
                        match loopcount {
                            0..=5 => {
                                Circle::new(Point::new(1, 53), 8)
                                    .into_styled(fill)
                                    .draw(&mut display).unwrap();
                            },
                            _ => {},
                        }
                    },
                    LoggingStatus::Stop => {
                    },
                }
                let cur_pos = 50;
                if lck.current.abs() < 0.001 {
                    Text::new(&format!("{:.0}uA", lck.current * 1000_000.0), Point::new(10, cur_pos), middle_style_white).draw(&mut display).unwrap();
                }
                else if lck.current.abs() < 1.0 {
                    Text::new(&format!("{:.0}mA", lck.current * 1000.0), Point::new(10, cur_pos), middle_style_white).draw(&mut display).unwrap();
                }
                else if lck.current > 32.0 {
                    Text::new("N/A", Point::new(10, cur_pos), middle_style_red).draw(&mut display).unwrap();
                }
                else {
                    Text::new(&format!("{:.1}A", lck.current), Point::new(10, cur_pos), middle_style_red).draw(&mut display).unwrap();
                }

                if lck.power < 1.0 {
                    Text::new(&format!("{:.0}mW", lck.power * 1000.0), Point::new(48, cur_pos), middle_style_white).draw(&mut display).unwrap();
                }
                else {
                    Text::new(&format!("{:.1}W", lck.power), Point::new(48, cur_pos), middle_style_red).draw(&mut display).unwrap();
                }
                Text::new(&format!("Int.{}ms", lck.interval), Point::new(10, 60), middle_style_white).draw(&mut display).unwrap();

                // Water mark of buffer
                let bar_len = (lck.buffer_water_mark * 95 / 100) as i32;
                Line::new(Point::new(0,63), Point::new(bar_len, 63)).into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 1)).draw(&mut display).unwrap();
                Triangle::new(Point::new(bar_len-2,61), Point::new(bar_len,63), Point::new(bar_len-2,63)).into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 1)).draw(&mut display).unwrap();

                match lck.wifi {
                    WifiStatus::Disconnected => {
                    },
                    WifiStatus::Connected => {
                        wifi_img.draw(&mut display).unwrap();
                    },
                }

                battery_queue[loopcount] = lck.battery;
                let mut battery_average = 0.0;
                for i in 0..10 {
                    battery_average += battery_queue[i];
                }
                battery_average = battery_average / 10.0;
                Text::new(&format!("{:.1}V", battery_average), Point::new(76, 60), small_style_white).draw(&mut display).unwrap();
                //                info!("battery_average: {}v", battery_average);
                match battery_level {
                    0 => {
                        if battery_average > 3.75 {
                            battery_level = 20;
                        }
                    },
                    20 => {
                        if battery_average > 3.85 {
                            battery_level = 40;
                        }
                        else if battery_average < 3.7 {
                            battery_level = 0;
                        }
                    },
                    40 => {
                        if battery_average > 3.95 {
                            battery_level = 60;
                        }
                        else if battery_average < 3.8 {
                            battery_level = 20;
                        }
                    },
                    60 => {
                        if battery_average > 4.05 {
                            battery_level = 80;
                        }
                        else if battery_average < 3.9 {
                            battery_level = 40;
                        }
                    },
                    80 => {
                        if battery_average > 4.15 {
                            battery_level = 100;
                        }
                        else if battery_average < 4.0 {
                            battery_level = 60;
                        }
                    }
                    100 => {
                        if battery_average > 4.55 {
                            battery_level = 200;
                        }
                        else if battery_average < 4.1 {
                            battery_level = 80;
                        }
                    },
                    200 => {
                        if battery_average < 4.5 {
                            battery_level = 100;
                        }
                    },
                    _ => {
                        battery_level = 0;
                    }
                }
                match battery_level {
                    0 => {
                        bat0_img.draw(&mut display).unwrap();
                    },
                    20 => {
                        bat20_img.draw(&mut display).unwrap();
                    },
                    40 => {
                        bat40_img.draw(&mut display).unwrap();
                    },
                    60 => {
                        bat60_img.draw(&mut display).unwrap();
                    },
                    80 => {
                        bat80_img.draw(&mut display).unwrap();
                    },
                    100 => {
                        bat100_img.draw(&mut display).unwrap();
                    },
                    200 => {
                        usbpwr_img.draw(&mut display).unwrap();
                    },
                    _ => {}
                }

                loopcount += 1;
                if loopcount == 10 {
                    loopcount = 0;
                }
                display.flush().unwrap();
                drop(lck);
                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    pub fn set_voltage(&mut self, vol: f32, cur: f32, power: f32)
    {
        let mut lck = self.txt.lock().unwrap();
        lck.voltage = vol;
        lck.current = cur;
        lck.power = power;
    }

    pub fn set_interval(&mut self, interval : u32)
    {
        let mut lck = self.txt.lock().unwrap();
        lck.interval = interval;
    }

    pub fn set_current_status(&mut self, status: LoggingStatus)
    {
        let mut lck= self.txt.lock().unwrap();
        lck.status = status;
    }

    pub fn set_wifi_status(&mut self, status: WifiStatus)
    {
        let mut lck= self.txt.lock().unwrap();
        lck.wifi = status;
    }

    pub fn set_err_message(&mut self, msg: String)
    {
        let mut lck = self.txt.lock().unwrap();
        lck.message = msg;
        lck.voltage = -999.0;
    }

    pub fn set_battery(&mut self, bat: f32){
        let mut lck = self.txt.lock().unwrap();
        lck.battery = bat;
    }

    pub fn set_buffer_watermark(&mut self, wm: u32){
        let mut lck = self.txt.lock().unwrap();
        lck.buffer_water_mark = wm;
    }
}
