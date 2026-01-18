use std::io;
use tokio::sync::mpsc;
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
pub trait Actor: Sized + Send + Sync {
    async fn run(_: (), ch_cont: &ChannelContainer, mut rec: mpsc::Receiver<Self>
    ) -> io::Result<()>;
}

#[derive(Default)]
pub struct ActorsContainer {
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
    pub async fn message<T: 'static + Any + Actor>(&self, msg: T) -> Result<(), ActorError> {
        if let Some(sender) = self.get_sender::<T>() {
            let _ = sender.send(msg).await
                .map_err(|_| ActorError::SendFailed);
            Ok(())
        }
        else {
            Err(ActorError::SenderNotFound)
        }
    }
    
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

#[macro_export]
macro_rules! spawn_actors {
    ($ch_cont:expr, $actors:expr, $($actor_type:ty),+ $(,)?) => {
        $(
            paste! {
                let ([<sender_ $actor_type:snake>], [<receiver_ $actor_type:snake>])
                    = mpsc::channel::<$actor_type>(1000);
            }
        )*
        $(
            paste! {
                $ch_cont.add_sender([<sender_ $actor_type:snake>]);
            }
        )*
        $(
            paste! {
                $actors.add_actor($ch_cont, [<receiver_ $actor_type:snake>]);
            }
        )*
                
    }
}
pub use spawn_actors;
