use std::io::Write;
use std::io::Read;
use std::thread::spawn;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::SocketAddr;
use clap::Parser;


fn handle_stream(mut stream: TcpStream, addr: SocketAddr, remote_host: &String) {
  println!("new connction from {}:{}", addr.ip(), addr.port());

  let mut remote = TcpStream::connect(remote_host).unwrap();

  let mut _remote = remote.try_clone().unwrap();
  let mut _stream = stream.try_clone().unwrap();
  let rh = String::from(remote_host);

  spawn(move || {
    loop {
      let mut _buf = [0; 1600];

      let n = match _remote.read(&mut _buf) {
        // socket closed
        Ok(n) if n == 0 => {
          println!("close connction ({})", rh);
          return;
        }
        Ok(n) => n,
        Err(e) => {
          eprintln!(
            "failed to read ({}); err = {:?}",
            rh,
            e
          );
          return;
        }
      };

      // Write the data back
      if let Err(e) = _stream.write_all(&_buf[0..n]) {
        eprintln!(
          "failed to write ({}); err = {:?}",
          rh,
          e
        );
        return;
      }
    }
  });

  loop {
    let mut _buf = [0; 1600];

    let n = match stream.read(&mut _buf) {
      // socket closed
      Ok(n) if n == 0 => {
        println!("close connction ({}:{})", addr.ip(), addr.port());
        return;
      }
      Ok(n) => n,
      Err(e) => {
        eprintln!(
          "failed to read ({}:{}); err = {:?}",
          addr.ip(),
          addr.port(),
          e
        );
        return;
      }
    };

    // Write the data back
    if let Err(e) = remote.write_all(&_buf[0..n]) {
      eprintln!(
        "failed to write ({}:{}); err = {:?}",
        addr.ip(),
        addr.port(),
        e
      );
      return;
    }
  }
}

#[derive(Parser)]
struct Args {
  /// port for incoming connection
  #[clap(value_parser)]
  port: u16,
  #[clap(value_parser)]
  remote_host: String,
  #[clap(value_parser)]
  remote_port: u16,
}

fn main() -> std::io::Result<()> {
  let args = Args::parse();

  let _listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], args.port)))?;

  println!("listening started on port {}, ready to accept.", args.port);

  loop {
    let (stream, addr) = _listener.accept()?;

    let rhost = format!("{}:{}",args.remote_host,args.remote_port);

    spawn(move || {
      handle_stream(stream, addr, &rhost);
    });
  }
}
