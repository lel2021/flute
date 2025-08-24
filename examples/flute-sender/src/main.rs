use flute::{
    core::UDPEndpoint,
    core::lct::Cenc,
    sender::{ObjectDesc, Sender},
};
use std::{net::UdpSocket, time::SystemTime};

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::builder().try_init().ok();

    // 从命令行参数获取目标地址
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("Send files over UDP/FLUTE using unicast");
        println!("Usage: {} destination_ip:port path/to/file1 path/to/file2 ...", args[0]);
        println!("Example: {} 172.18.202.15:3400 file1.txt file2.jpg", args[0]);
        std::process::exit(0);
    }

    let dest = &args[1];

    // 使用单播地址而不是组播地址
    let endpoint = UDPEndpoint::new(None, "0.0.0.0".to_owned(), 3400);

    log::info!("Create UDP Socket");

    // 绑定到所有接口
    let udp_socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    // 设置发送缓冲区大小
    // udp_socket.set_send_buffer_size(1024 * 1024).unwrap();

    log::info!("Create FLUTE Sender");
    let tsi = 1;
    let mut sender = Sender::new(endpoint, tsi, &Default::default(), &Default::default());

    log::info!("Sending to {}", dest);
    udp_socket.connect(dest).expect("Connection failed");

    for file in &args[2..] {
        let path = std::path::Path::new(file);

        if !path.is_file() {
            log::error!("{} is not a file", file);
            continue; // 跳过无效文件而不是退出
        }

        log::info!("Insert file {} to FLUTE sender", file);
        let obj = ObjectDesc::create_from_file(
            path,
            None,
            "application/octet-stream",
            true,
            1,
            None,
            None,
            None,
            None,
            Cenc::Null,
            true,
            None,
            true,
        )
            .unwrap();
        sender.add_object(0, obj).unwrap();
    }

    log::info!("Publish FDT update");
    sender.publish(SystemTime::now()).unwrap();

    let mut sent_packets = 0;
    while let Some(pkt) = sender.read(SystemTime::now()) {
        match udp_socket.send(&pkt) {
            Ok(_) => {
                sent_packets += 1;
                if sent_packets % 100 == 0 {
                    log::info!("Sent {} packets", sent_packets);
                }
            }
            Err(e) => {
                log::error!("Failed to send packet: {}", e);
            }
        }
        // 稍微减慢发送速度以避免拥塞
        std::thread::sleep(std::time::Duration::from_micros(10));
    }

    log::info!("File transfer completed. Total packets sent: {}", sent_packets);
}