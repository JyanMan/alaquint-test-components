use alaquint_comps::socket::Socket;
use std::io;
use static_cell::StaticCell;
use std::sync::Arc;

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("start main");

    tokio::spawn(read_socket()).await?;

    println!("end main");
    Ok(())
}

async fn read_socket() -> io::Result<()> {
    println!("yow");
    let addr = "127.0.0.1:2000";
    let sock = match Socket::new(addr).await {
            Ok(res) => {
                println!("socket created");
                res
            },
            Err(res) => {
                println!("failed to create socket: {:?}", res);
                return Err(res)
            }
        }
    ;
    
    let result = sock.read_bytes(20).await?;
    if result.len() != 0 {
        println!("res {:?}", result);
    }
    else {
        println!("no bytes to read at addr: {}", addr);
    }
    Ok(())
}
