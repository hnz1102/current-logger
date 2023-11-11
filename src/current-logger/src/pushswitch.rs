use log::*;
use std::{thread, time::Duration, sync::Arc, sync::Mutex};
use esp_idf_hal::{gpio::*, task};
use esp_idf_sys::{tskTaskControlBlock, TaskHandle_t};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::time::SystemTime;

const PUSH_NOTIFICATION_GPIO20: u32 = 1;
const PUSH_NOTIFICATION_GPIO21: u32 = 2;

type PINDRIVER20 = Box<PinDriver<'static, esp_idf_hal::gpio::Gpio20, esp_idf_hal::gpio::Input>>;
type PINDRIVER21 = Box<PinDriver<'static, esp_idf_hal::gpio::Gpio21, esp_idf_hal::gpio::Input>>;

struct ButtonState {
    startstop: bool,
    send: bool,
}

pub struct PushSwitch {
    state: Arc<Mutex<ButtonState>>
}

impl PushSwitch {
    pub fn new() -> PushSwitch {
        PushSwitch { state: Arc::new(Mutex::new(
            ButtonState { startstop: false, send: false })) }
    }

    pub fn start(&mut self,
            mut gpio20_sig : PINDRIVER20,
            mut gpio21_sig : PINDRIVER21)
    {
        let state = self.state.clone();
        let _th = thread::spawn(move || {
            info!("Start Switch Read Thread.");
            let task_handle_20: AtomicPtr<tskTaskControlBlock> = AtomicPtr::new(std::ptr::null_mut());
            let ptr_20: TaskHandle_t = task::current().unwrap();
            task_handle_20.store(ptr_20, Ordering::Relaxed);
            let task_handle_21: AtomicPtr<tskTaskControlBlock> = AtomicPtr::new(std::ptr::null_mut());
            let ptr_21: TaskHandle_t = task::current().unwrap();
            task_handle_21.store(ptr_21, Ordering::Relaxed);
            
            gpio20_sig.set_pull(Pull::Up).unwrap();
            gpio21_sig.set_pull(Pull::Up).unwrap();
            gpio20_sig.set_interrupt_type(InterruptType::NegEdge).unwrap();
            gpio21_sig.set_interrupt_type(InterruptType::NegEdge).unwrap();
            unsafe {
                gpio20_sig.subscribe(move || {
                    task::notify(task_handle_20.load(Ordering::Relaxed), PUSH_NOTIFICATION_GPIO20);
                }).unwrap();
                gpio21_sig.subscribe(move || {
                    task::notify(task_handle_21.load(Ordering::Relaxed), PUSH_NOTIFICATION_GPIO21);
                }).unwrap();
            }        
            let now = SystemTime::now();
            let mut gpio20_trigger_time : u32 = 0;
            let mut gpio21_trigger_time : u32 = 0;
            loop {
                let res = task::wait_notification(Some(Duration::from_millis(10)));
                let mut lck = state.lock().unwrap();
                match res {
                    Some(PUSH_NOTIFICATION_GPIO20) => {
//                        info!("PUSH_NOTIFICATION_GPIO20");
                        if gpio20_trigger_time == 0 {
                            gpio20_trigger_time = now.elapsed().unwrap().as_millis() as u32;
                        }
                    },
                    Some(PUSH_NOTIFICATION_GPIO21) => {
//                       info!("PUSH_NOTIFICATION_GPIO21");
                       if gpio21_trigger_time == 0 {
                           gpio21_trigger_time = now.elapsed().unwrap().as_millis() as u32;
                       }
                    },
                    _ => {
                        if gpio20_trigger_time > 0 && gpio20_trigger_time + 300 < now.elapsed().unwrap().as_millis() as u32 {
                            lck.startstop = true;
                            gpio20_trigger_time = 0;
                        }
                        if gpio21_trigger_time > 0 && gpio21_trigger_time + 300 < now.elapsed().unwrap().as_millis() as u32 {
                            lck.send = true;
                            gpio21_trigger_time = 0;
                        }
                    },
                }
                drop(lck);                
            }
        });
    }

    pub fn get_gpio_state(&mut self, gpio: u32) -> bool
    {
        let mut lock= self.state.lock().unwrap();
        match gpio {
            20 => {
                let ret = lock.startstop;
                lock.startstop = false;
                ret
            },
            21 => {
                let ret = lock.send;
                lock.send = false;
                ret
            },
            _ => {
                false
            }
        }
    }

}
