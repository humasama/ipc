use socket2::{Domain, SockAddr, Socket, Type};
use std::{
    env, fs,
    io::{Read, Write},
    path::PathBuf,
    thread,
    time::Duration,
};

use anyhow::Result;

const DATA: &[u8] = b"hello world";

fn client() -> Result<()> {
    let path: PathBuf = "/tmp/socket/unix".into();

    let mut sock = Socket::new(Domain::UNIX, Type::SEQPACKET, None)?;
    let saddr = SockAddr::unix(path).unwrap();
    sock.connect(&saddr)?;
    println!("Client: connect to server.");

    loop {
        let mut buf = [0; DATA.len()];
        match sock.read(&mut buf) {
            Ok(num) => {
                if num < DATA.len() {
                    println!("Failed to receive all data ({:?})", buf);
                    break;
                }
                // println!("Receive {:?} from server", buf);
            }
            Err(_) => todo!(),
        }
    }
    Ok(())
}

fn server() -> Result<()> {
    let mut path = env::temp_dir();
    path.push("socket");
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.push("unix");

    let saddr = SockAddr::unix(&path)?;
    let listener = Socket::new(Domain::UNIX, Type::SEQPACKET, None)?;
    listener.bind(&saddr)?;
    listener.listen(10)?;

    let (mut conn, addr) = listener.accept().unwrap();
    println!("Server: receive a connection ({:?})", addr);
    loop {
        match conn.write(DATA) {
            Ok(num) => {
                if num < DATA.len() {
                    println!("Fail to send all bytes (success={})", num);
                    break;
                }
            }
            Err(_) => (),
        }
    }
    Ok(())
}

fn main() {
    let handle1 = thread::spawn(|| {
        let _ = server();
    });

    thread::sleep(Duration::from_millis(20));

    let handle2 = thread::spawn(|| {
        let _ = client();
    });
    handle1.join().unwrap();
    handle2.join().unwrap();
}

mod test {
    #[test]
    fn uds() {
        use socket2::{Domain, SockAddr, Socket, Type};
        use std::{
            env, fs,
            io::{Read, Write},
        };

        const DATA: &[u8] = b"hello world";
        let mut path = env::temp_dir();
        path.push("socket2");
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path.push("unix");

        let addr = SockAddr::unix(path).unwrap();

        let listener = Socket::new(Domain::UNIX, Type::STREAM, None).unwrap();
        listener.bind(&addr).unwrap();
        listener.listen(10).unwrap();

        let mut a = Socket::new(Domain::UNIX, Type::STREAM, None).unwrap();
        a.connect(&addr).unwrap();
        let mut b = listener.accept().unwrap().0;

        let _ = a.write(DATA).unwrap();
        let mut buf = [0; DATA.len() + 1];
        let n = b.read(&mut buf).unwrap();
        assert_eq!(n, DATA.len());
        assert_eq!(&buf[..n], DATA);
    }
}