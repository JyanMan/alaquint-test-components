use std::io;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use std::any::{TypeId, Any};
use std::collections::HashMap;
use std::boxed::Box;
use async_trait::async_trait;
use paste::paste;

#[derive(Debug)]
pub enum ActorError {
    SenderNotFound,
    SendFailed
}

impl std::fmt::Display for ActorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ActorError::SenderNotFound => write!(f, "Sender not found"),
            ActorError::SendFailed => write!(f, "Failed to send message"),
        }
    }
}

#[async_trait]
pub trait Actor: Sized + Send + Sync + 'static {
    // type Msg;
    
    async fn run(
        // mut self,
        ch_cont: &ChannelContainer,
        mut rec: mpsc::Receiver<Self>
    ) -> io::Result<()>;
}

#[derive(Default)]
pub struct ActorsContainer {
    actors: HashMap<TypeId, JoinHandle<Result<(), io::Error>>>,
}

impl ActorsContainer {
    pub fn create_channel<T: Actor>(buf_size: usize) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
        mpsc::channel::<T>(buf_size)
    }

    pub fn add_actor<T: Actor>(&mut self, ch_cont: &'static ChannelContainer, rec: mpsc::Receiver<T>) {
        // self.senders.insert(TypeId::of::<T>(), Box::new(sender));
        // let (sender, receiver) = mpsc::channel::<T>(1000);
        // let new_self = T::default();
        let task_spawn = tokio::spawn(T::run(ch_cont, rec));
        self.actors.insert(TypeId::of::<T>(), task_spawn);
    } 

    pub async fn await_actors(&mut self) -> io::Result<()> {
        for (_type, task_handle) in self.actors.drain() {
            match task_handle.await? {
                Ok(()) => (),
                Err(err) => {
                    println!("err {:?}", err);
                }
            }
        }
        Ok(())
    }
}


#[derive(Default)]
pub struct ChannelContainer {
    senders: HashMap<TypeId, Box<dyn Any>>,
}

pub trait MessageRequest: 'static + Send + Sync {
    type Response;
    fn create_channel() -> ( oneshot::Sender<Self::Response>, oneshot::Receiver<Self::Response> );
}

// traits needed to be safe between threads
unsafe impl Send for ChannelContainer {}
unsafe impl Sync for ChannelContainer {}

impl ChannelContainer {
    // pub async fn request_test<T: Actor, Msg: MessageRequest>(&self, msg: Msg) -> io::Result<()> {
    //     let (tx, rx) = Msg::create_channel();
    //     if let Some(sender) = self.get_sender::<T>() {
    //         let _ = sender.send(msg).await;
    //         Ok(())
    //     }
        
    //     Ok(())
    // }
    
    pub async fn request<T: Actor>(&self, msg: T) -> io::Result<()> {
        if let Some(sender) = self.get_sender::<T>() {
            let _ = sender.send(msg).await;
            Ok(())
        }
        else {
            Err(io::Error::new(io::ErrorKind::NotFound, "sender not found"))
        }
        // Ok(())
        // else {
        //     Err(ActorError::SenderNotFound)
        // }
    }

    pub async fn message<T: Actor>(&self, msg: T) -> Result<(), ActorError> {
        if let Some(sender) = self.get_sender::<T>() {
            let _ = sender.send(msg).await
                .map_err(|_| ActorError::SendFailed);
            Ok(())
        }
        else {
            Err(ActorError::SenderNotFound)
        }
    }
    
    pub fn add_sender<T: Actor>(&mut self, sender: mpsc::Sender<T>) {
        self.senders.insert(TypeId::of::<T>(), Box::new(sender));
    } 

    pub fn get_sender<T: Actor>(&self) -> Option<&mpsc::Sender<T>> {
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


#[macro_export]
macro_rules! spawn_actors {
    ($ch_cont:expr, $actors:expr, $($actor_type:ty),+ $(,)?) => {
        $(
            paste! {
                let ([<sender_ $actor_type:snake>], [<receiver_ $actor_type:snake>])
                    = ActorsContainer::create_channel::<$actor_type>(1000);
            }
        )*
        $(
            paste! {
                $ch_cont.add_sender::<$actor_type>([<sender_ $actor_type:snake>]);
            }
        )*
        $(
            paste! {
                $actors.add_actor::<$actor_type>($ch_cont, [<receiver_ $actor_type:snake>]);
            }
        )*
                
    }
}
pub use spawn_actors;
