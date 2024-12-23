use tokio::net::TcpStream;
use std::net::{IpAddr, SocketAddr};
use tokio::time::{timeout, Duration};
use structopt::StructOpt;
use std::sync::Arc;

/// Struktur untuk mengatur argumen command line
#[derive(StructOpt, Debug)]
struct Opt {
    /// IP target untuk discan
    #[structopt(short, long)]
    ip: IpAddr,

    /// Rentang awal port
    #[structopt(short = "s", long, default_value = "1")]
    start_port: u16,

    /// Rentang akhir port
    #[structopt(short = "e", long, default_value = "1024")]
    end_port: u16,

    /// Mode verbose
    #[structopt(short, long)]
    verbose: bool,

    /// Batas jumlah task paralel
    #[structopt(short = "t", long, default_value = "100")]
    max_tasks: usize,
}

/// Fungsi untuk melakukan scanning pada satu port
async fn scan_port(ip: IpAddr, port: u16, verbose: bool) -> bool {
    let socket_addr = SocketAddr::new(ip, port);
    if verbose {
        println!("Scanning port {}...", port);
    }

    let connection = timeout(Duration::from_secs(1), TcpStream::connect(socket_addr)).await;

    match connection {
        Ok(Ok(_)) => true,  // Jika koneksi berhasil
        _ => false,         // Timeout atau gagal koneksi
    }
}

#[tokio::main]
async fn main() {
    // Parsing argumen command line
    let opt = Opt::from_args();
    let port_range = opt.start_port..=opt.end_port;

    println!("Scanning ports on {} from {} to {}...", opt.ip, opt.start_port, opt.end_port);

    // Gunakan Arc untuk berbagi semaphore antar tugas
    let semaphore = Arc::new(tokio::sync::Semaphore::new(opt.max_tasks));
    let mut tasks = vec![];

    for port in port_range {
        let permit = semaphore.clone().acquire_owned().await.unwrap(); // Menggunakan Arc
        let ip = opt.ip;
        let verbose = opt.verbose;

        let task = tokio::spawn(async move {
            let result = scan_port(ip, port, verbose).await;
            drop(permit); // Kembalikan slot setelah task selesai
            if result {
                println!("Port {} is open", port);
            }
        });

        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }

    println!("Scan completed.");
}
