

use std::{thread, time::Duration};
use esp_idf_hal::{gpio::*, prelude::*, spi, i2c};
use esp_idf_hal::delay::BLOCK;
use esp_idf_hal::peripherals::Peripherals;
use embedded_hal::spi::MODE_0;
use log::*;
use std::time::SystemTime;
use esp_idf_hal::adc::config::Config as AdcConfig;
use esp_idf_hal::adc::AdcChannelDriver;
use esp_idf_hal::adc::AdcDriver;
use esp_idf_hal::adc::Atten11dB;

mod pushswitch;
mod displayctl;
mod currentlogs;
mod wifi;
mod transfer;

use pushswitch::PushSwitch;
use displayctl::{DisplayPanel, LoggingStatus, WifiStatus};
use currentlogs::{CurrentRecord, CurrentLog};
use transfer::Transfer;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("")]
    http_server: &'static str,
}

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // Initialize nvs
    unsafe {
        esp_idf_sys::nvs_flash_init();
    }
    // Peripherals Initialize
    let peripherals = Peripherals::take().unwrap();
    
    // Display SPI
    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio8;
    let sdo  = peripherals.pins.gpio9;
    let sdi_not_used : Option<Gpio2> = None;
    let cs_not_used : Option<Gpio2> = None;
    let dc = PinDriver::output(peripherals.pins.gpio10)?;
    let rst = PinDriver::output(peripherals.pins.gpio1)?;
    let spi_config = spi::SpiConfig::new().baudrate(4.MHz().into()).data_mode(MODE_0);

    let spi_driver = spi::SpiDriver::new(
        spi,
        sclk,
        sdo,
        sdi_not_used,
        spi::Dma::Disabled,
    ).unwrap();
    
    let spi_device = spi::SpiDeviceDriver::new(spi_driver, cs_not_used, &spi_config)?;
    let mut dp = DisplayPanel::new();
    dp.start(spi_device, dc, rst);

    // Current/Voltage
    let i2c = peripherals.i2c0;
    let scl = peripherals.pins.gpio4;
    let sda = peripherals.pins.gpio3;
    let config = i2c::I2cConfig::new().baudrate(400.kHz().into());
    let mut i2cdrv = i2c::I2cDriver::new(i2c, sda, scl, &config)?;

    // SHUNT_CAL
    let current_lsb = 16.384 / 524_288.0;
    let shunt_cal_val = 13107.2 * current_lsb * 1000_000.0 * 0.010;
    let shunt_cal = shunt_cal_val as u32;
    info!("current_lsb={:?} shunt_cal_val={:?}", current_lsb, shunt_cal_val);
    i2cdrv.write(0x40, &[0x02u8; 1], BLOCK).expect("INA228 I2C Write Error");
    let mut shunt_cal_buf = [0u8; 2];
    shunt_cal_buf[0] = (shunt_cal >> 8) as u8;
    shunt_cal_buf[1] = (shunt_cal & 0xFF) as u8;
    i2cdrv.write(0x40, &shunt_cal_buf, BLOCK)?;

    // PushSW
    let interval_select_pin   = peripherals.pins.gpio20;
    let startstop_pin   = peripherals.pins.gpio21;
    let interval_select_sig   = Box::new(PinDriver::input(interval_select_pin)?);
    let startstop_sig   = Box::new(PinDriver::input(startstop_pin)?);
    let mut psw         = PushSwitch::new();
    psw.start(interval_select_sig, startstop_sig);
    let mut interval_led   = PinDriver::input_output(peripherals.pins.gpio6)?;
    let mut startstop_led   = PinDriver::input_output(peripherals.pins.gpio7)?;
    startstop_led.set_low()?;
    interval_led.set_low()?;

    // Temperature Logs
    let mut clogs = CurrentRecord::new();

    // WiFi
    let wifi_enable : bool;
    let wifi = wifi::wifi_connect(peripherals.modem, CONFIG.wifi_ssid, CONFIG.wifi_psk);
    match wifi {
        Ok(_) => { wifi_enable = true; },
        Err(e) => { info!("{:?}", e); wifi_enable = false }
    }
    let mut txd =  Transfer::new(CONFIG.http_server.to_string());
    txd.start()?;
    
    // ADC GPIO0
    let mut adc = AdcDriver::new(peripherals.adc1, &AdcConfig::new().calibration(true))?;
    let mut adc_pin: esp_idf_hal::adc::AdcChannelDriver<'_, Gpio0, Atten11dB<_>> =
        AdcChannelDriver::new(peripherals.pins.gpio0)?;

    // loop
    let mut measurement_count : u32 = 0;
    let mut logging_start = false;
    let mut measuring_interval : u32 = 4;    // 4ms interval time when it starts
    dp.set_interval(measuring_interval+1);
    let mut start_logging_time = SystemTime::now();
    let mut next_time = start_logging_time.checked_add(Duration::from_millis(measuring_interval as u64)).expect("no next_time");
    let mut measurement_light = false;
    loop {
        thread::sleep(Duration::from_millis(1));

        let interval_select_btn = psw.get_gpio_state(20);
        let start_stop_btn = psw.get_gpio_state(21);
        if start_stop_btn == true {
            if logging_start == true {
                // to Stop
                logging_start = false;
                // clogs.dump();
                // clogs.clear();
            }
            else {
                // to Start
                logging_start = true;
                measurement_count = 0;
                info!("Logging and Sending Start..");
                clogs.clear();
                start_logging_time = SystemTime::now();
            }
        }
        if interval_select_btn == true {
            measuring_interval = match measuring_interval {
                4   => 9,
                9   => 49,
                49  => 99,
                99  => 499,
                499 => 999,
                _ => 4,
            };
            dp.set_interval(measuring_interval+1);
            measurement_light = false;
            measurement_count = 0;
            start_logging_time = SystemTime::now();
            next_time = start_logging_time.checked_add(Duration::from_millis(measuring_interval as u64)).expect("no next_time");
        }

        if wifi_enable == false{
            dp.set_wifi_status(WifiStatus::Disconnected);
        }
        else {
            dp.set_wifi_status(WifiStatus::Connected);
        }

        if logging_start == true {
            startstop_led.set_high()?;
            dp.set_current_status(LoggingStatus::Start);
        }
        else {
            startstop_led.set_low()?;
            dp.set_current_status(LoggingStatus::Stop);
        }


        let intime = next_time.duration_since(SystemTime::now());
        match intime {
            Ok(_v) => { continue },
            Err(_e) => { },
        }
        measurement_count += 1;
        next_time = start_logging_time.checked_add(Duration::from_millis((measuring_interval * measurement_count) as u64)).expect("no next_time");
//        let duration = SystemTime::now();

        if measurement_light == false {
            interval_led.set_high()?;
            measurement_light = true;
        }
        else {
            interval_led.set_low()?;
            measurement_light = false;
        }

       // Read Current/Voltage
        let mut vbus_buf  = [0u8; 3];
        let mut data = CurrentLog::default();
        // Timestamp
        data.clock = start_logging_time.elapsed().unwrap().as_millis() as u32;

        i2cdrv.write(0x40, &[0x05u8; 1], BLOCK)?;
        match i2cdrv.read(0x40, &mut vbus_buf, BLOCK){
            Ok(_v) => {
                // info!("vbus={:?}", vbus_buf);
                let vbus = ((((vbus_buf[0] as u32) << 16 | (vbus_buf[1] as u32) << 8 | (vbus_buf[2] as u32)) >> 4) as f32 * 193.3125) / 1000_000.0;
                data.voltage = vbus; // V
            },
            Err(e) => {
                info!("{:?}", e);
                dp.set_err_message(format!("{:?}", e));
            }
        }
        let mut curt_buf  = [0u8; 3];
        i2cdrv.write(0x40, &[0x07u8; 1], BLOCK)?;
        match i2cdrv.read(0x40, &mut curt_buf, BLOCK) {
            Ok(_v) => {
                let current_reg : f32;
                if curt_buf[0] & 0x80 == 0x80 {
                    current_reg = (0x100000 - (((curt_buf[0] as u32) << 16 | (curt_buf[1] as u32) << 8 | (curt_buf[2] as u32)) >> 4)) as f32 * -1.0;
                }
                else {
                    current_reg = (((curt_buf[0] as u32) << 16 | (curt_buf[1] as u32) << 8 | (curt_buf[2] as u32)) >> 4) as f32;
                }
                let current = current_lsb * current_reg;
                data.current = current;   // A
                // info!("curt={:?} {:?}A", curt_buf, data.current);
            },
            Err(e) => {
                info!("{:?}", e);
                dp.set_err_message(format!("{:?}", e));
            }
        }
        let mut power_buf = [0u8; 3];
        i2cdrv.write(0x40, &[0x08u8; 1], BLOCK)?;
        match i2cdrv.read(0x40, &mut power_buf, BLOCK) {
            Ok(_v) => {
                // info!("power={:?}", power_buf);
                let power_reg = ((power_buf[0] as u32) << 16 | (power_buf[1] as u32) << 8 | (power_buf[2] as u32)) as f32;
                let power = 3.2 * current_lsb * power_reg;        
                data.power = power;   // W
            },
            Err(e) => {
                info!("{:?}", e);
                dp.set_err_message(format!("{:?}", e));
            }
        }
        // battery voltage 
        data.battery =  adc.read(&mut adc_pin).unwrap() as f32 * 2.0 / 1000.0;
        dp.set_battery(data.battery);
        dp.set_voltage(data.voltage, data.current, data.power);
        if logging_start {
            clogs.record(data);
        }
        let current_record = clogs.get_size();
        if current_record >= 4095 {
            logging_start = false;  // Auto stop logging if buffer is full.
        }
        dp.set_buffer_watermark((current_record as u32) * 100 / 4095);

        if wifi_enable == true && current_record > 0 {
            let logs = clogs.get_all_data();
            let txcount = txd.set_transfer_data(logs);
            if txcount > 0 {
                clogs.remove_data(txcount);
            }
        }
//        info!("duration = {:?}", duration.elapsed().unwrap().as_millis());
    }
}
