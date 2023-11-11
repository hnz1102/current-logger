use log::*;
use std::{thread, time::Duration, sync::Arc, sync::Mutex};
use esp_idf_hal::task;
use std::io::prelude::*;
use std::io::Error;
use std::net::{TcpStream, Shutdown};

const MEASUREMENT : &str  = "currentlogger";
const POINT_TAG : &str    = "currentch1";

use anyhow::Result;

use crate::CurrentLog;

const HTTP_HEADER : &str = "Content-Type: application/json\r\nAcept: */*\r\nUser-Agent: temp-logger";

struct TransferData {
    body: String,
    txreq: bool,
}

pub struct Transfer {
    data: Arc<Mutex<TransferData>>,
    server: String,
}

impl Transfer {
    pub fn new(server: String) -> Self {
        Transfer { data: Arc::new(Mutex::new(
            TransferData { body: "".to_string(), txreq: false })),
            server: server}
    }

    pub fn start(&mut self) -> Result<(), Error>
    {
        let data = self.data.clone();
        let server = self.server.clone();
        let _th = thread::spawn(move || -> anyhow::Result<()> {
            info!("Start transfer thread.");
            loop {
                task::wait_notification(Some(Duration::from_millis(100)));
                let mut lck = data.lock().unwrap();
                if lck.txreq == false {
                    drop(lck);
                    continue;
                }
                let request = format!("POST / HTTP/1.1\r\nHost: {}\r\n{}\r\nContent-Length:{}\r\n\r\n{}\r\n",
                    &server, HTTP_HEADER, lck.body.len(), lck.body);
                drop(lck);                
                let ret = Self::transfer(&server, request);
                lck = data.lock().unwrap();
                match ret {
                    Ok(()) => { lck.txreq = false; },
                    Err(e) => { info!("{}", e) },
                }
                drop(lck);
            }
        });

        Ok(())
    }

    fn transfer(server: &String, request: String) -> Result<(), std::io::Error>
    {
        let mut stream = TcpStream::connect(&server)?;
        stream.write(request.as_bytes())?;
        let mut rcvbuf = [0u8; 1024];
        stream.read(&mut rcvbuf)?;        
        stream.shutdown(Shutdown::Both).expect("shutdown call failed");
        Ok(())
    }


    pub fn set_transfer_data(&mut self, data: &Vec<CurrentLog>) -> usize
    {
        if data.len() == 0 {
            return 0;
        }
        let mut lck = self.data.lock().unwrap();
        if lck.txreq == true {
            // There is sending data in buffer.
            return 0;
        }
        lck.body = format!("[ ");
        let mut count = 0;
        for it in data {
            lck.body.push_str(
                &format!("{{ \"measurement\": \"{}\", \"tag\": \"{}\", \"timestamp\": {}, \"current\": {:.5}, \"voltage\": {:.5},  \"power\": {:.5}, \"bat\": {:.2} }}",
                MEASUREMENT,
                POINT_TAG,
                it.clock,
                it.current,
                it.voltage,
                it.power,
                it.battery
            ));
            count += 1;
            if count == 64 {
                info!("Chunk data");
                break;
            }
            if data.len() != count {
                lck.body.push_str(",");
            }
        }
        lck.body.push_str("]");
        lck.txreq = true;
        count as usize
    }
}
