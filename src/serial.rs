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

// WARNING: code is still inefficient due to locks on each use
#[allow(unused)]
impl Serial {
    pub fn new(path: &str, baud_rate: u32) -> io::Result<Self> {
        let port = tokio_serial::new(path, baud_rate)
            .open_native_async(); 
        if let Ok(port) = port {
            Ok(Self(port))
        }
        else {
            Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "Port not connected"))
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

    pub fn try_read_bytes(&mut self, buff_size: usize) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0u8; buff_size];
        let port = &mut self.0;
        let _ = port.try_read(&mut buffer[..])?;
        Ok(buffer)
    }

    pub fn try_write_bytes(&mut self, to_send: &[u8]) -> io::Result<()> {
        self.0.try_write(to_send)?;
        Ok(())
    }
    
    pub async fn read_bytes(&mut self, buff_size: usize) -> io::Result<Vec<u8>> {
        use tokio::io::AsyncReadExt;
        let mut buffer = vec![0u8; buff_size];
        let port = &mut self.0;
        let _ = port.read(&mut buffer[..]).await;
        Ok(buffer)
    }

    pub async fn write_bytes(&mut self, to_send: &[u8]) -> io::Result<()> {
        use tokio::io::AsyncWriteExt;
        self.0.write_all(to_send).await?;
        Ok(())
    }
}
