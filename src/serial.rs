use bevy_ecs::prelude::*;
use std::sync::Mutex;
use std::vec::Vec;

/// `usage`:
/// ```
/// let serial = SerialPort::new("/dev/ttyUSB0", 115200);
/// let bytes = serial.read_bytes();
/// ```
/// `manual usage of serialport crate`
/// ```
/// serial.port_ref().lock().read();
/// serial.port_ref().lock().some_other_func();
/// ```
type MutexSerialPort = Mutex<Box<dyn serialport::SerialPort>>;

#[derive(Component)]
pub struct Serial(Option<MutexSerialPort>);

#[allow(unused)]
impl Serial {
    pub fn new(path: &str, baud_rate: u32) -> Self {
        let port = serialport::new(path, baud_rate).open(); 
        if let Ok(port) = port {
            Self(Some(Mutex::new(port)))
        }
        else {
            Self(None)
        }
    }

    pub fn init_port(&mut self, path: &str, baud_rate: u32) -> &Self {
        let port = serialport::new(path, baud_rate).open(); 
        if let Ok(port) = port {
            self.0 = Some(Mutex::new(port))
        }
        self
    }

    pub fn port_ref(&self) -> Option<&MutexSerialPort> {
        self.0.as_ref()
    } 

    pub fn port_mut(&mut self) -> Option<&mut MutexSerialPort> {
        self.0.as_mut()
    } 
    
    pub fn read_bytes(&self, buff_size: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; buff_size];
        if let Some(port) = &self.0 {
            let _ = port.lock().unwrap().read(&mut buffer[..]).unwrap();
        }
        buffer
    }

    // this uses write_all... use .write for more control
    pub fn write_bytes(&self, to_send: &[u8]) -> Result<(), std::io::Error> {
        if let Some(port) = &self.0 {
            port.lock().unwrap().write_all(to_send);
            Ok(())
        }
        else {
            Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "Port not initialized"))
        }
    }

    // using write_bytes allows for this check at the same time
    pub fn port_connected(&self) -> bool {
        self.0.is_some()
    }
}
