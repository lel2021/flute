use flute::{
    core::UDPEndpoint,
    receiver::{writer, MultiReceiver},
};
use std::rc::Rc;

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::builder().try_init().ok();
    // 从命令行参数获取监听端口
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Save FLUTE objects received via unicast");
        println!("Usage: {} path/to/destination_folder [port]", args[0]);
        println!("Default port: 3400");
        std::process::exit(0);
    }

    let port = if args.len() > 2 {
        args[2].parse().unwrap_or(3400)
    } else {
        3400
    };

    // 使用单播端点
    let endpoint = UDPEndpoint::new(None, "0.0.0.0".to_string(), port);

    let dest_dir = std::path::Path::new(&args[1]);
    if !dest_dir.is_dir() {
        log::error!("{:?} is not a directory", dest_dir);
        std::process::exit(-1);
    }

    log::info!("Create FLUTE receiver, writing objects to {:?}", dest_dir);

    let enable_md5_check = true;
    let writer = Rc::new(writer::ObjectWriterFSBuilder::new(dest_dir, enable_md5_check).unwrap());
    let mut receiver = MultiReceiver::new(writer, None, false);

    // 创建普通UDP socket而不是组播socket
    let socket = std::net::UdpSocket::bind(format!("0.0.0.0:{}", port))
        .expect("Failed to bind UDP socket");

    // 设置接收缓冲区大小
    // socket.set_recv_buffer_size(1024 * 1024).unwrap();

    log::info!("Listening on port {} for FLUTE data", port);

    let mut buf = [0; 204800];
    let mut received_packets = 0;
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, src)) => {
                received_packets += 1;
                if received_packets % 100 == 0 {
                    log::info!("Received {} packets from {}", received_packets, src);
                }

                let now = std::time::SystemTime::now();
                if let Err(e) = receiver.push(&endpoint, &buf[..n], now) {
                    log::error!("Error processing packet: {:?}", e);
                }
                receiver.cleanup(now);
            }
            Err(e) => {
                log::error!("Failed to receive data: {:?}", e);
            }
        }
    }
}