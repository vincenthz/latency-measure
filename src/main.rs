use clap::Parser;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::time::{Duration, SystemTime};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
enum Subcommand {
    Client(ClientArgs),
    Server(ServerArgs),
}

#[derive(Parser, Debug)]
pub struct ClientArgs {
    #[clap(long, short)]
    pub port: u16,
    #[clap(long, short)]
    pub client_port: u16,
    #[clap(long, short)]
    pub destination: String,
    #[clap(long, default_value_t = false)]
    pub udp: bool,
    #[clap(long, default_value_t = 3)]
    pub tries: u32,
}

#[derive(Parser, Debug)]
pub struct ServerArgs {
    #[clap(long, short)]
    pub port: u16,
    #[clap(long, default_value_t = false)]
    pub udp: bool,
    #[clap(long, default_value_t = 3)]
    pub tries: u32,
}

fn ser(s: Duration) -> [u8; 12] {
    let mut out = [0; 12];
    let x = s.as_secs().to_le_bytes();
    let y = s.subsec_nanos().to_le_bytes();
    out[0..8].copy_from_slice(&x);
    out[8..12].copy_from_slice(&y);
    out
}

fn ser2(s1: Duration, s2: Duration) -> [u8; 24] {
    let mut out = [0; 24];
    let x = s1.as_secs().to_le_bytes();
    let y = s1.subsec_nanos().to_le_bytes();
    out[0..8].copy_from_slice(&x);
    out[8..12].copy_from_slice(&y);

    let x = s2.as_secs().to_le_bytes();
    let y = s2.subsec_nanos().to_le_bytes();
    out[12..20].copy_from_slice(&x);
    out[20..24].copy_from_slice(&y);
    out
}

fn unser(buf: [u8; 12]) -> Duration {
    let s1 = u64::from_le_bytes(buf[0..8].try_into().unwrap());
    let n1 = u32::from_le_bytes(buf[8..12].try_into().unwrap());
    let d1 = Duration::new(s1, n1);
    d1
}

fn unser2(buf: [u8; 24]) -> (Duration, Duration) {
    let s1 = u64::from_le_bytes(buf[0..8].try_into().unwrap());
    let s2 = u64::from_le_bytes(buf[12..20].try_into().unwrap());
    let n1 = u32::from_le_bytes(buf[8..12].try_into().unwrap());
    let n2 = u32::from_le_bytes(buf[20..24].try_into().unwrap());
    let d1 = Duration::new(s1, n1);
    let d2 = Duration::new(s2, n2);
    (d1, d2)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.subcommand {
        Subcommand::Client(args) => {
            let addr = format!("{}:{}", args.destination, args.port);

            let addr_client = format!("0.0.0.0:{}", args.client_port);
            //let mut stream = TcpStream::connect(addr)?;
            let mut stream = UdpSocket::bind(addr_client)?;
            let mut addr = addr.to_socket_addrs().unwrap();
            let addr = addr.next().unwrap();

            for _ in 0..args.tries {
                let now = SystemTime::now();
                let d = now.duration_since(SystemTime::UNIX_EPOCH)?;
                let buf = ser(d);
                //stream.write_all(&buf)?;
                stream.send_to(&buf, addr)?;

                let now2 = SystemTime::now();
                let mut buf = [0u8; 24];
                //stream.read_exact(&mut buf)?;
                let (amt, _) = stream.recv_from(&mut buf)?;
                if amt < 24 {
                    panic!("arg {}", amt)
                }
                let df = now2.duration_since(SystemTime::UNIX_EPOCH)?;
                let (d1, _d2) = unser2(buf);
                let x = d1 - d;
                let x2 = d1 - df;
                println!("{:?} {:?}", x, x2);
            }
            Ok(())
        }
        Subcommand::Server(args) => {
            let addr = format!("0.0.0.0:{}", args.port);
            //let listener = TcpListener::bind(format!("0.0.0.0:{}", args.port))?;
            //let (mut connection, client_addr) = listener.accept()?;
            let mut connection = UdpSocket::bind(addr)?;
            for _ in 0..args.tries {
                let mut buf = [0u8; 12];
                //connection.read_exact(&mut buf)?;
                let (amt, src) = connection.recv_from(&mut buf)?;
                if amt < 12 {
                    panic!("arg {}", amt)
                }
                let d = unser(buf);
                let t = SystemTime::now();
                let d = t.duration_since(SystemTime::UNIX_EPOCH)?;
                let x = ser2(d, d);
                //connection.write_all(&x)?;
                connection.send_to(&x, src)?;
            }

            Ok(())
        }
    }
}
