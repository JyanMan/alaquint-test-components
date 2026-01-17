use std::io;
use tokio::sync::mpsc;
use std::any::{TypeId, Any};
use std::vec::Vec;
use std::collections::HashMap;
use std::boxed::Box;
use std::sync::Arc;
use static_cell::StaticCell;
use std::ops::DerefMut;
use alaquint_comps::serial::Serial;

#[derive(PartialEq, Eq, Hash, Clone)]
enum LidarReader { }

#[derive(PartialEq, Eq, Hash, Clone)]
enum Motor {
    GetRpm
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


#[derive(Default)]
struct ChannelContainer {
    senders: HashMap<TypeId, Box<dyn Any>>,
}

// traits needed to be safe between threads
unsafe impl Send for ChannelContainer {}
unsafe impl Sync for ChannelContainer {}

impl ChannelContainer {
    pub fn add_sender<T: 'static + Any>(&mut self, sender: mpsc::Sender<T>) {
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

    // let mut senders: HashMap<MessageType, mpsc::Sender<Message>> = HashMap::new();
    let (send_to_lidar_reader, rec_lidar_reader) = mpsc::channel::<LidarReader>(1000);
    let (send_to_motor, rec_motor) = mpsc::channel::<Motor>(1000);
    let (send_to_pid, rec_pid) = mpsc::channel::<PidSystem>(1000);

    ch_cont.add_sender(send_to_lidar_reader);
    ch_cont.add_sender(send_to_motor);
    ch_cont.add_sender(send_to_pid);

    let actors = [
        tokio::spawn(actor_lidar_reader_debug(ch_cont, rec_lidar_reader)),
        tokio::spawn(actor_motor(ch_cont, rec_motor)),
        tokio::spawn(actor_pid_system(ch_cont, rec_pid))
    ];
    for actor in actors {
        let _ = actor.await?;
    }
    
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
    const PACKET_HEADER: [u8; 2] = [0x02, 0x03]
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
