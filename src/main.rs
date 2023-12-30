use std::net::Ipv4Addr;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::signal;
use tokio::time::{self, Duration};

async fn ingress(server_address: &str) {
    let listener = TcpListener::bind(server_address).await.unwrap();
    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok ((socket, _)) => {
                    tokio::spawn(async move {
                        process_ingress(socket).await;
                    });
                    },
                    Err(err) => {
                        println!("Error: {}", err);
                        return;
                    }
                }
            }
        }
    }
}

async fn process_ingress(mut socket: TcpStream) {
    let mut buffer = [0u8; 4096];
    let mut interval = time::interval(Duration::from_secs(10));
    // println!("Accepting connection: {:?}", socket);
    loop {
        tokio::select! {
            res = socket.read(&mut buffer) => {
                match res {
                    Ok(0) => {
                        println!("Ingress end");
                        return;
                    },
                    Ok(n) => {
                        // println!("{}", n);
                    },
                    Err(e) => {
                        eprintln!("Error while reading incoming connection: {e}.");
                        return;
                    }
                }
            }
            _ = interval.tick() => {
                if let Err(e) = socket.write_all(b".").await {
                    eprintln!("Failed to write to socket: {:?}.", e);
                    break;
                }
            }
        }
    }
}

async fn egress(server_address: &str) {
    const MAX_CONN: usize = 512768;
    let (tx, mut rx) = tokio::sync::mpsc::channel::<(usize, usize)>(MAX_CONN);
    let mut dyn_cnt = 0;
    let mut offset = 0;
    for conn in 0..MAX_CONN {
        dyn_cnt += 1;
        if dyn_cnt > 10000 {
            offset += 1;
            dyn_cnt = 0;
            println!("Going to next ip, {offset}");
        }
        tokio::spawn(process_egress(
            conn,
            tx.clone(),
            server_address.to_string(),
            offset,
        ));
    }
    println!("Connection pool initialized.");
    loop {
        tokio::select! {
            msg = rx.recv() => {
                let (id, offset) = msg.unwrap();
                println!("Lost egress connection {:?}, respawing.", id);
                tokio::spawn(process_egress(id, tx.clone(), server_address.to_string(), offset));
            }
        }
    }
}

fn make_ip(offset: u32) -> std::net::IpAddr {
    let base = 2130706433u32; // 127.0.0.1
    std::net::IpAddr::V4(Ipv4Addr::from(base.wrapping_add(offset)))
}

async fn process_egress(
    id: usize,
    channel: tokio::sync::mpsc::Sender<(usize, usize)>,
    server_address: String,
    offset: usize,
) {
    let mut buffer = [0u8; 4096];
    let mut interval = time::interval(Duration::from_secs(10));

    let src_ip = make_ip(offset as u32);

    let socket = TcpSocket::new_v4().unwrap();
    socket.bind(SocketAddr::new(src_ip, 0)).unwrap();

    let server_addr = server_address.parse::<SocketAddr>().unwrap();

    match socket.connect(server_addr).await {
        Ok(mut socket) => {
            // println!(
            //     "Successfully connected to {} as {:?}",
            //     &server_address, &socket
            // );
            loop {
                tokio::select! {
                    res = socket.read(&mut buffer) => {
                        match res {
                            Ok(0) => {
                                println!("Egress end.");
                                return;
                            },
                            Ok(n) => {
                                // println!("{}", n);
                            },
                            Err(e) => {
                                eprintln!("Error while reading incoming connection: {e}.");
                                return;
                            }
                        }
                    }
                    _ = interval.tick() => {
                        if let Err(e) = socket.write_all(b".").await {
                            eprintln!("Failed to write to socket: {:?}.", e);
                            break;
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Failed to connect to {}: {}.", &server_address, e),
    }
    channel.send((id, offset)).await.unwrap();
}

#[tokio::main]
async fn main() {
    let server_address = "127.0.0.1:6379";

    let ingress_handle = tokio::spawn(ingress(server_address));
    let egress_handle = tokio::spawn(egress(server_address));
    let ctrl_c_handle = signal::ctrl_c();
    tokio::select! {
        _ = ingress_handle => {
            println!("Ingress task shutdown.");
        },
        _ = egress_handle => {
            println!("Egress task shutdown.");
        },
        _ = ctrl_c_handle => {
            println!("Exiting.");
        }
    }
}
