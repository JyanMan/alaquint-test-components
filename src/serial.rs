use std::sync::Mutex;
use std::vec::Vec;
// use std::io;
use tokio_serial::{SerialPort, SerialPortBuilderExt, SerialStream};
use tokio::io;

/// ```
/// let serial = SerialPort::new("/dev/ttyUSB0", 115200);
/// let bytes = serial.read_bytes().await?;
/// ```
pub struct Serial(SerialStream);

#[allow(unused)]
impl Serial {
    pub fn new(path: &str, baud_rate: u32) -> io::Result<Self> {
        let port = tokio_serial::new(path, baud_rate)
            .open_native_async(); 
        match port {
            Ok(port) => Ok(Self(port)),
            Err(err) => Err(err.into())
        }
    }

    pub fn init_port(&mut self, path: &str, baud_rate: u32) -> &Self {
        let port = tokio_serial::new(path, baud_rate).
            open_native_async(); 
        if let Ok(port) = port {
            self.0 = port
        }
        self
    }

    pub fn port_ref(&self) -> &SerialStream {
        &self.0
    } 

    pub fn port_mut(&mut self) -> &mut SerialStream {
        &mut self.0
    } 

    pub fn try_read_bytes(&mut self, buff_size: usize) -> io::Result<(Vec<u8>, usize)> {
        let mut buffer = vec![0u8; buff_size];
        let port = &mut self.0;
        let n = port.try_read(&mut buffer[..])?;
        Ok((buffer, n))
    }

    pub fn try_write_bytes(&mut self, to_send: &[u8]) -> io::Result<usize> {
        self.0.try_write(to_send)
    }

    pub fn bytes_to_read(&self) -> u32 {
        self.0.bytes_to_read().unwrap()
    }
    
    pub async fn read_bytes(&mut self, buff_size: usize) -> io::Result<(Vec<u8>, usize)> {
        use tokio::io::AsyncReadExt;
        let mut buffer = vec![0u8; buff_size];
        let port = &mut self.0;
        let n = port.read(&mut buffer[..]).await?;
        Ok((buffer, n))
    }

    pub async fn write_bytes(&mut self, to_send: &[u8]) -> io::Result<()> {
        use tokio::io::AsyncWriteExt;
        self.0.write_all(to_send).await
    }
}
