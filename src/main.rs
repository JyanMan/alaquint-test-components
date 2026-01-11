use bevy_ecs::prelude::*;
use test_comp::prelude::Serial;

fn main() {
    println!("test component");
    let mut world = World::new();

    let _ = world.spawn(Serial::new("/dev/ttyUSB0", 115200)).id();

    let mut schedule = Schedule::default();
    schedule.add_systems(read_serial);

    for _ in 1..5 {
        schedule.run(&mut world);
    }
}

fn read_serial(query: Query<&Serial>) {
    for serial in query {
        if !serial.port_connected() {
            println!("port is not connected");
            continue;
        }
        let read_bytes = serial.read_bytes(20);
        println!("bytes read: {:?}", read_bytes);
    }
}
