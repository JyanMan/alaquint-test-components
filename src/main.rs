use std::io;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
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

#[derive(PartialEq, Eq, Hash, Clone)]
enum LidarReader { }

#[async_trait]
impl Actor for LidarReader {
    async fn run(
        _: (),
        send: &ChannelContainer,
        mut rec: mpsc::Receiver<LidarReader>,
    ) -> io::Result<()> {
        let _ = send.message(Motor::GetRpm).await;
        Ok(())
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
enum Motor {
    GetRpm
}

#[async_trait]
impl Actor for Motor {
    async fn run(
        _: (),
        send: &ChannelContainer,
        mut rec: mpsc::Receiver<Motor>,
    ) -> io::Result<()> {

        let motor_msg = rec.recv().await;
        motor_msg.unwrap().parse();

        Ok(())
    }
}

/// `NOTE` all methods are to not be used and are private
/// this is to test the feasabily of an action-oriented design
/// every call must be used via the enum values
impl Motor {
    fn get_rpm(&self) {
        println!("SOMEBODY ASKED FOR THE RPM");
    }
    fn parse(&self) {
        match self {
            Motor::GetRpm => self.get_rpm()
            
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
enum PidSystem {}

#[async_trait]
impl Actor for PidSystem {
    async fn run(_: (), ch_cont: &ChannelContainer, mut rec: mpsc::Receiver<PidSystem>
    ) -> io::Result<()> {
        const PACKET_HEADER: [u8; 2] = [0x02, 0x03];
        if let Ok(mut serial) = Serial::new("/dev/ttyUSB0", 115200) {
            let res_bytes = serial.read_bytes(20).await?;
            println!("read data: {:?}", res_bytes);
            serial.write_bytes(&PACKET_HEADER).await?;
        }
        Ok(())
    }
}


static CH_CONT: StaticCell<ChannelContainer> = StaticCell::new();

#[tokio::main]
async fn main() -> io::Result<()> {
    let ch_cont = CH_CONT.init(ChannelContainer::default());
    let mut actors = ActorsContainer::default();

    spawn_actors!(
        ch_cont,
        actors,
        LidarReader,
        Motor
    );

    actors.await_actors().await?;
    
    Ok(())
}
