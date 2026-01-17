// use bevy_ecs::prelude::*;
use alaquint_comps::serial::Serial;
use alaquint_comps::ecs::{ECS, UpdateFn};

fn main() {
    println!("test component");
    // let mut world = World::new();
    let mut ecs = ECS::new();

    ecs.register_system_async(read_serial);

    // ecs.register_system_update(Box::new(|ecs: &mut ECS, delta_time: f32| {
    //     read_serial(ecs, delta_time);
    // }));

    for _ in 1..5 {
        ecs.call_update_systems(1.0 / 60.0);
    }

    // if let Ok(serial) = Serial::new("/dev/ttyUSB0", 115200) {
    //     let _ = world.spawn(serial).id();
    // }

    // let mut schedule = Schedule::default();
    // schedule.add_systems(read_serial);

    // for _ in 1..5 {
    //     schedule.run(&mut world);
    // }
}

async fn read_serial(ecs: &mut ECS) {
    println!("hello broda");
    // for serial in query {
    //     let read_bytes = serial.read_bytes(20);
    //     println!("bytes read: {:?}", read_bytes);
    // }
}
