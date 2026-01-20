use std::io;
use std::str;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::Duration;
use std::any::{TypeId, Any};
use std::vec::Vec;
use std::collections::HashMap;
use std::boxed::Box;
use std::sync::Arc;
use static_cell::StaticCell;
use std::ops::DerefMut;
use alaquint_comps::serial::Serial;
use alaquint_comps::actor_system::{
    ChannelContainer,
    Actor,
    ActorsContainer,
    spawn_actors
};
use async_trait::async_trait;
use paste::paste;
use alaquint_proc_macros::RequestMsg;
// extern crate proc_macro;
// use proc_macro::TokenStream;

// #[proc_macro_derive(RequestMsg, attributes(request))]
// pub fn derive_request_msg(main_t: TokenStream, send_t: TokenStream) -> TokenStream {
//     "fn \"{item}\"() -> (oneshot::Sender<\"{send_t}\">, oneshot::Sender<\"{send_t}\">) {
//         oneshot::channel::<\"{send_t}\">()
//     }".parse().unwrap()
// }

#[derive(PartialEq, Eq, Hash, Clone)]
enum LidarReaderMsg { }

#[derive(Default)]
struct LidarReader;

#[async_trait]
impl Actor for LidarReader {
    type Msg = LidarReaderMsg;
    
    async fn run(
        self,
        send: &ChannelContainer,
        _rec: mpsc::Receiver<LidarReaderMsg>,
    ) -> io::Result<()> {
        let _ = send.message::<Motor>(MotorMsg::SetName(String::from("BRODA"))).await;
        // let (sender, rec) = oneshot::channel::<i32>();
        // let _ = send.request::<Motor>(MotorMsg::GetRpm{reply: sender}).await;
        let rpm = MotorMsg::get_rpm(send).await?;
        // MotorMsg::get_rpm();

        println!("lez go: {:?}", rpm);
        // match MotorMsg::req_rpm(send).await {
        //     Ok(v) => println!("received rpm: {:?}", v),
        //     Err(err) => println!("unable to receive rpm: {:?}", err)
        // }

        Ok(())
    }
}

// #[derive(Clone)]
#[derive(RequestMsg)]
enum MotorMsg {
    #[request(response = i32)]
    GetRpm{reply: oneshot::Sender<i32>},
    SetName(String),
}

impl MotorMsg {
    // pub async fn get_rpm(send: &ChannelContainer) -> io::Result<i32> {
    //     let (tx, rx) = oneshot::channel();
    //     let _ = send.request::<Motor>(MotorMsg::GetRpm{reply: tx}).await;
    //     match rx.await {
    //        Ok(res) => Ok(res),
    //        Err(e) => {
    //             Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
    //         }
    //     }
    // }
}


struct Motor {
    name: String,
    rpm: i32
}

/// `NOTE` all methods are to not be used and are private
/// this is to test the feasabily of an action-oriented design
/// every call must be used via the enum values
impl Default for Motor {
    fn default() -> Self {
        Self {
            name: String::from("whatsoeverr"),
            rpm: 10,
        }
    }
}

impl Motor {
    fn get_rpm(&self) -> i32 {
        self.rpm
    }
}

#[async_trait]
impl Actor for Motor {
    type Msg = MotorMsg;
    
    async fn run(
        mut self,
        send: &ChannelContainer,
        mut rec: mpsc::Receiver<MotorMsg>,
    ) -> io::Result<()> {

        loop {
            let motor_msg = rec.recv().await;
            match motor_msg.unwrap() {
                MotorMsg::GetRpm{reply} => {
                    println!("SOMEBODY ASKED FOR THE RPM");
                    let _ = reply.send(self.get_rpm());
                    // send.message::<LidarReader>()
                },
                MotorMsg::SetName(new_name) => {
                    self.name = new_name;
                    println!("set new name to: {}", self.name);
                }
            }
            
        }
        // Ok(())
    }
}

#[derive(Default, PartialEq, Eq, Hash, Clone)]
pub enum PidSystem {
    #[default]
    Hello
}

#[derive(Clone)]
pub enum PidCmd {
    Idk = 0x6769,
}

impl PidCmd {
    pub fn lsb(&self) -> u8 {
        self.clone() as u8
    }
    pub fn msb(&self) -> u8 {
        ((self.clone() as u16) >> 8) as u8
    }
}

// #[derive(Default)]
// pub struct PidSystem {}

#[async_trait]
impl Actor for PidSystem {
    type Msg = PidSystem;
    
    async fn run(self, ch_cont: &ChannelContainer, mut rec: mpsc::Receiver<PidSystem>) -> io::Result<()> {

        const PACKET_SIZE: usize = 20;
        const PACKET_HEADER: [u8; 2] = [0xAA, 0x55];
        const PORT: &str = "/dev/ttyUSB0";
        const BAUD: u32 = 115200;

        let mut serial = Serial::new(PORT, BAUD)?;
        let mut write_buf = Vec::with_capacity(PACKET_SIZE);

        loop {
            let cmd = PidCmd::Idk;
            let left_rpm: f32 = 33.0;
            let right_rpm: f32 = 66.0;

            write_buf.clear();
            write_buf.extend_from_slice(&PACKET_HEADER);

            write_buf.push(cmd.lsb());
            write_buf.push(cmd.msb());

            write_buf.extend_from_slice(&right_rpm.to_le_bytes());
            write_buf.extend_from_slice(&left_rpm.to_le_bytes());
            
            serial.write_bytes(&write_buf[..]).await?;

            let (res_bytes, res) = serial.read_bytes(20).await?;
            match str::from_utf8(&res_bytes[..res]) {
                Ok(res) => print!("{}", res),
                Err(_) => println!("[software] failed to parse bytes")
            }
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {

    static CH_CONT: StaticCell<ChannelContainer> = StaticCell::new();
    let ch_cont = CH_CONT.init(ChannelContainer::default());
    let mut actors = ActorsContainer::default();

    spawn_actors!(
        ch_cont,
        actors,
        // PidSystem
        LidarReader,
        Motor
    );

    actors.await_actors().await?;
    
    Ok(())
}
