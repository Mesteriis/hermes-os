use std::{
    env,
    net::{SocketAddr, TcpStream},
    process::ExitCode,
    time::Duration,
};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

fn usage() -> &'static str {
    "usage: hermes-development-platform-runtime probe --postgres-address HOST:PORT --nats-address HOST:PORT"
}

fn parse_address(flag: &str, value: Option<String>) -> Result<SocketAddr, String> {
    value
        .ok_or_else(|| format!("{flag} requires HOST:PORT"))?
        .parse()
        .map_err(|_| format!("{flag} must be a numeric HOST:PORT socket address"))
}

fn probe(name: &str, address: SocketAddr) -> Result<(), String> {
    TcpStream::connect_timeout(&address, CONNECT_TIMEOUT)
        .map(|_| ())
        .map_err(|error| format!("{name} at {address} is unavailable: {error}"))
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    if args.next().as_deref() != Some("probe") {
        eprintln!("{0}", usage());
        return ExitCode::from(2);
    }

    let postgres_flag = args.next();
    let postgres = if postgres_flag.as_deref() == Some("--postgres-address") {
        parse_address("--postgres-address", args.next())
    } else {
        Err("expected --postgres-address".to_owned())
    };
    let nats_flag = args.next();
    let nats = if nats_flag.as_deref() == Some("--nats-address") {
        parse_address("--nats-address", args.next())
    } else {
        Err("expected --nats-address".to_owned())
    };
    if args.next().is_some() {
        eprintln!("{0}", usage());
        return ExitCode::from(2);
    }

    let (postgres, nats) = match (postgres, nats) {
        (Ok(postgres), Ok(nats)) => (postgres, nats),
        (Err(error), _) | (_, Err(error)) => {
            eprintln!("{error}\n{0}", usage());
            return ExitCode::from(2);
        }
    };

    if let Err(error) = probe("PostgreSQL", postgres) {
        eprintln!("{error}");
        return ExitCode::from(1);
    }
    if let Err(error) = probe("NATS", nats) {
        eprintln!("{error}");
        return ExitCode::from(1);
    }

    println!("development-platform-runtime: PostgreSQL and NATS TCP endpoints reachable");
    ExitCode::SUCCESS
}
