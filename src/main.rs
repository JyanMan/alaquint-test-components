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
use async_trait::async_trait;


#[async_trait]
pub trait Actor: Sized + Send + Sync {
    async fn run(_: (), ch_cont: &ChannelContainer, mut rec: mpsc::Receiver<Self>
    ) -> io::Result<()>;
}

#[derive(PartialEq, Eq, Hash, Clone)]
enum LidarReader { }

#[async_trait]
impl Actor for LidarReader {
    async fn run(
        _: (),
        send: &ChannelContainer,
        mut rec: mpsc::Receiver<LidarReader>,
    ) -> io::Result<()> {
        let motor = send.get_sender::<Motor>().unwrap();
        send.send_data(motor, Motor::GetRpm).await?;
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
enum PidSystem {
    
}

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

#[derive(Default)]
struct ActorsContainer {
    actors: HashMap<TypeId, JoinHandle<Result<(), io::Error>>>,
}

impl ActorsContainer {

    pub fn add_actor<T: 'static + Any + Actor>(&mut self, ch_cont: &'static ChannelContainer, rec: mpsc::Receiver<T>) {
        // self.senders.insert(TypeId::of::<T>(), Box::new(sender));
        // let (sender, receiver) = mpsc::channel::<T>(1000);
        let task_spawn = tokio::spawn(T::run((), ch_cont, rec));
        self.actors.insert(TypeId::of::<T>(), task_spawn);
    } 

    pub async fn await_actors(&mut self) -> io::Result<()> {
        for (_type, task_handle) in self.actors.drain() {
            let _ = task_handle.await?;
        }
        Ok(())
    }
}


#[derive(Default)]
pub struct ChannelContainer {
    senders: HashMap<TypeId, Box<dyn Any>>,
}

// traits needed to be safe between threads
unsafe impl Send for ChannelContainer {}
unsafe impl Sync for ChannelContainer {}

impl ChannelContainer {
    pub fn add_sender<T: 'static + Any + Actor>(&mut self, sender: mpsc::Sender<T>) {
        self.senders.insert(TypeId::of::<T>(), Box::new(sender));
    } 

    pub fn get_sender<T: 'static + Any>(&self) -> Option<&mpsc::Sender<T>> {
        self.senders
            .get(&TypeId::of::<T>())
            .and_then(|s| s.downcast_ref::<mpsc::Sender<T>>())
    }

    pub async fn send_data<T: 'static + Any>(
        &self,
        target: &mpsc::Sender<T>,
        msg: T
    ) -> io::Result<()> {
        let _ = target.send(msg).await;
        Ok(())
    }
}

static CH_CONT: StaticCell<ChannelContainer> = StaticCell::new();

#[tokio::main]
async fn main() -> io::Result<()> {
    // let ch_cont = Arc::new(ChannelContainer::default());
    let ch_cont = CH_CONT.init(ChannelContainer::default());
    let mut actors = ActorsContainer::default();

    // let mut senders: HashMap<MessageType, mpsc::Sender<Message>> = HashMap::new();
    let (send_to_lidar_reader, rec_lidar_reader) = mpsc::channel::<LidarReader>(1000);
    let (send_to_motor, rec_motor) = mpsc::channel::<Motor>(1000);

    ch_cont.add_sender(send_to_lidar_reader);
    ch_cont.add_sender(send_to_motor);

    actors.add_actor(ch_cont, rec_lidar_reader);
    actors.add_actor(ch_cont, rec_motor);

    actors.await_actors().await?;

    
    Ok(())
}

async fn actor_motor(
    send: &ChannelContainer,
    mut rec: mpsc::Receiver<Motor>,
) -> io::Result<()> {

    let motor_msg = rec.recv().await;
    motor_msg.unwrap().parse();

    Ok(())
}

async fn actor_pid_system(
    send: &ChannelContainer,
    mut rec: mpsc::Receiver<PidSystem>,
) -> io::Result<()> {
    const PACKET_HEADER: [u8; 2] = [0x02, 0x03];
    if let Ok(mut serial) = Serial::new("/dev/ttyUSB0", 115200) {
        let res_bytes = serial.read_bytes(20).await?;
        println!("read data: {:?}", res_bytes);
        serial.write_bytes(&PACKET_HEADER).await?;
    }
    Ok(())
}

async fn actor_lidar_reader_debug(
    send: &ChannelContainer,
    mut rec: mpsc::Receiver<LidarReader>,
) -> io::Result<()> {
    let motor = send.get_sender::<Motor>().unwrap();
    send.send_data(motor, Motor::GetRpm).await?;
    Ok(())
}
