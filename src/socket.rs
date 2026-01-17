use tokio::net::UdpSocket;
use std::io;

#[derive(Debug)]
pub struct Socket {
    socket: UdpSocket
}

impl Socket {
    pub async fn new(addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(Self{socket})
    }

    pub async fn read_bytes(&self, buf_size: usize) -> io::Result<Vec<u8>> {
        self.socket.readable().await?;
        let mut buf = vec![0u8; buf_size];
        let (len, addr) = self.socket.recv_from(&mut buf[..]).await?;
        println!("{:?} bytes received from {:?}", len, addr);
        Ok(buf)

    }

    pub async fn send_bytes(&self, buf_size: usize) -> Result<(), io::Error> {
        let buf = vec![0u8; buf_size];
        if let Ok(addr) = self.socket.local_addr() {
            let len = self.socket.send_to(&buf[..], addr).await?;
            println!("{:?} bytes sent", len);
            Ok(())
        }
        else {
            Err(io::Error::new(io::ErrorKind::AddrNotAvailable, ""))
        }
    }
}







