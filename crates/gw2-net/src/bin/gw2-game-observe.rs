//! Offline build-209026 game-handshake observer. See docs/re/209026-game-handshake.md.
//! Uses an explicitly supplied local DH identity; captures decrypted bytes privately.
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bytes::{Bytes, BytesMut};
use gw2_net::{
    ServerCodec,
    dh::{ByteOrder, ParamBlob, ServerKey, session_secret},
};
use num_bigint::BigUint;
use tokio_util::codec::{Decoder, Encoder};

#[path = "game_observe/auth.rs"]
mod auth;
#[path = "game_observe/mission.rs"]
mod mission;
#[path = "game_observe/preamble.rs"]
mod preamble;
use preamble::Preamble;

#[cfg(test)]
#[path = "game_observe/tests.rs"]
mod tests;

const CAPTURE_LIMIT: usize = 1024 * 1024;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn private_file(path: &Path) -> io::Result<File> {
    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    opts.open(path)
}

fn read_exact_file(path: &Path, size: usize) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    File::open(path)?
        .take((size + 1) as u64)
        .read_to_end(&mut data)?;
    if data.len() != size {
        return Err(invalid("wrong identity file length"));
    }
    Ok(data)
}

/// The client idles on the auth connection between login and Play (news
/// panel, User Agreement, character select); run 59 shows a 30 s timeout drops
/// it and forces a re-login. The Play and mission phases (the latter waits
/// through map loading) therefore wait longer, but still bounded.
fn read_timeout(long_phase: bool) -> Duration {
    Duration::from_secs(if long_phase { 600 } else { 30 })
}

/// Opt-in synthetic fixtures for one connection. All default to off.
#[derive(Default)]
struct Fixtures<'a> {
    reply: Option<&'a Bytes>,
    auth_body: Option<auth::ReplyFixture>,
    play: Option<auth::PlayRoute>,
    mission_map_key: Option<u32>,
    population: Option<mission::PopulationFixture>,
    /// Service mode: create no capture or preamble files. The auth stream can
    /// carry account strings typed into the client, so a server facing a
    /// person must run with this set.
    no_capture: bool,
}

fn observe(
    mut stream: TcpStream,
    key: &ServerKey,
    log: &Path,
    fixtures: Fixtures,
    preamble: Preamble,
) -> io::Result<()> {
    let Fixtures {
        reply,
        auth_body,
        play,
        mission_map_key,
        population,
        no_capture,
    } = fixtures;
    let mut mission = match (mission_map_key.map(mission::MissionPhase::new).transpose()?, population) {
        (Some(phase), Some(fixture)) => Some(phase.with_population(fixture)?),
        (phase, _) => phase,
    };
    let mut auth = match (auth_body.map(auth::AuthProbe::new), play) {
        (Some(probe), Some(route)) => Some(probe.with_play_route(route)?),
        (probe, _) => probe,
    };
    let play_enabled = auth.as_ref().is_some_and(auth::AuthProbe::play_enabled);
    stream.set_read_timeout(Some(read_timeout(play_enabled || mission.is_some())))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;
    let preamble = preamble.read(&mut stream)?;
    let mut header = [0u8; 2];
    stream.read_exact(&mut header)?;
    let len = usize::from(header[1]);
    if header[0] != 0 || !(6..=66).contains(&len) || (len - 2) % 4 != 0 {
        return Err(invalid("invalid client key record"));
    }
    let mut public = vec![0u8; len - 2];
    stream.read_exact(&mut public)?;
    let shared = key
        .shared(&BigUint::from_bytes_le(&public))
        .ok_or_else(|| invalid("degenerate client public value"))?;
    let mut block = [0u8; 20];
    getrandom::fill(&mut block).map_err(io::Error::other)?;
    let mut codec = ServerCodec::new(&session_secret(&shared, &block));
    let mut capture: Box<dyn Write> = if no_capture {
        Box::new(io::sink())
    } else {
        let mut prefix_capture = private_file(&log.with_extension("preamble.bin"))?;
        prefix_capture.write_all(&preamble)?;
        prefix_capture.flush()?;
        Box::new(private_file(log)?)
    };
    stream.write_all(&[1, 22])?;
    stream.write_all(&block)?;
    eprintln!("Game key reply sent; client public length {}", public.len());
    if let Some(reply) = reply {
        let mut wire = BytesMut::new();
        codec.encode(reply.clone(), &mut wire)?;
        stream.write_all(&wire)?;
        eprintln!("Explicit game reply sent: {} plaintext bytes", reply.len());
    }
    if let Some(mission) = &mission {
        let mut wire = BytesMut::new();
        codec.encode(mission.opening(), &mut wire)?;
        stream.write_all(&wire)?;
        eprintln!("Mission phase sent server 1063 MapData and 1060 LoadMap");
    }
    let mut chunk = [0u8; 8192];
    let mut total = 0;
    // The 1 MiB cap bounds capture files; without capture the session is
    // bounded only by the read timeout.
    while no_capture || total < CAPTURE_LIMIT {
        let available = if no_capture {
            chunk.len()
        } else {
            chunk.len().min(CAPTURE_LIMIT - total)
        };
        let n = match stream.read(&mut chunk[..available]) {
            Ok(n) => n,
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                break;
            }
            Err(e) => return Err(e),
        };
        if n == 0 {
            break;
        }
        let mut pending = BytesMut::from(&chunk[..n]);
        if let Some(decoded) = codec.decode(&mut pending)? {
            capture.write_all(&decoded)?;
            capture.flush()?;
            total += decoded.len();
            eprintln!("Decrypted game bytes: {} (total {total})", decoded.len());
            if let Some(auth) = &mut auth
                && let Some(reply) = auth.push(&decoded)?
            {
                let mut wire = BytesMut::new();
                codec.encode(reply, &mut wire)?;
                stream.write_all(&wire)?;
                eprintln!("Explicit correlated auth fixture sent");
            }
            if let Some(mission) = &mut mission {
                for seen in mission.push(&decoded) {
                    if let Some(reply) = mission.respond(&seen)? {
                        let mut wire = BytesMut::new();
                        codec.encode(reply, &mut wire)?;
                        stream.write_all(&wire)?;
                    }
                }
            }
        }
    }
    eprintln!("Game capture complete: {total} bytes");
    Ok(())
}

/// Loopback is always allowed. Any other address is a service listener: it
/// must run credential-safe (`--no-capture`) and name a specific interface,
/// never the unspecified address.
fn bind_allowed(listen: &SocketAddr, no_capture: bool) -> bool {
    listen.ip().is_loopback() || (no_capture && !listen.ip().is_unspecified())
}

fn main() -> io::Result<()> {
    let mut listen: SocketAddr = "127.1.0.2:6112".parse().unwrap();
    let (mut params, mut private, mut logs) = (None, None, None);
    let mut reply = None;
    let mut auth_body = None;
    let mut character = None;
    let mut preamble = Preamble::Auth;
    let mut play_address: Option<SocketAddr> = None;
    let mut mission_map_key: Option<u32> = None;
    let mut population_record: Option<Vec<u8>> = None;
    let mut no_capture = false;
    let mut spawn: Option<[f32; 3]> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--auth-character-record" => {
                if character.is_some() {
                    return Err(invalid("only one character record fixture is supported"));
                }
                let path = args
                    .next()
                    .ok_or_else(|| invalid("missing character record file"))?;
                let mut bytes = Vec::new();
                File::open(path)?
                    .take((auth::CHARACTER_BODY_LIMIT + 1) as u64)
                    .read_to_end(&mut bytes)?;
                character = Some(Bytes::from(bytes));
            }
            "--mission-preamble" => preamble = Preamble::Mission,
            "--mission-map-key" => {
                mission_map_key = Some(
                    args.next()
                        .ok_or_else(|| invalid("missing mission map key"))?
                        .parse()
                        .map_err(|_| invalid("invalid mission map key"))?,
                )
            }
            "--no-capture" => no_capture = true,
            "--mission-population" => {
                let path = args
                    .next()
                    .ok_or_else(|| invalid("missing population character record"))?;
                let mut bytes = Vec::new();
                File::open(path)?
                    .take((auth::CHARACTER_BODY_LIMIT + 1) as u64)
                    .read_to_end(&mut bytes)?;
                population_record = Some(bytes);
            }
            "--mission-spawn" => {
                let text = args.next().ok_or_else(|| invalid("missing spawn X,Y,Z"))?;
                let parts: Vec<f32> = text
                    .split(',')
                    .map(str::parse)
                    .collect::<Result<_, _>>()
                    .map_err(|_| invalid("invalid spawn X,Y,Z"))?;
                spawn = Some(
                    parts
                        .try_into()
                        .map_err(|_| invalid("spawn needs exactly X,Y,Z"))?,
                );
            }
            "--auth-play-route" => {
                play_address = Some(
                    args.next()
                        .ok_or_else(|| invalid("missing play-route address"))?
                        .parse()
                        .map_err(|_| invalid("invalid play-route address"))?,
                )
            }
            "--listen" => {
                listen = args
                    .next()
                    .ok_or_else(|| invalid("missing listen address"))?
                    .parse()
                    .map_err(|_| invalid("invalid listen address"))?
            }
            "--params" => params = args.next().map(PathBuf::from),
            "--private" => private = args.next().map(PathBuf::from),
            "--log" => logs = args.next().map(PathBuf::from),
            "--reply-frame" | "--auth-reply-body" => {
                let path = args.next().ok_or_else(|| invalid("missing reply file"))?;
                let mut bytes = Vec::new();
                File::open(path)?.take(65536).read_to_end(&mut bytes)?;
                if bytes.is_empty() || bytes.len() > 65535 {
                    return Err(invalid("reply must contain 1..65535 plaintext bytes"));
                }
                if arg == "--reply-frame" {
                    reply = Some(Bytes::from(bytes));
                } else {
                    if bytes.len() > 65000 {
                        return Err(invalid("auth fixture body exceeds bound"));
                    }
                    auth_body = Some(Bytes::from(bytes));
                }
            }
            _ => {
                return Err(invalid(
                    "usage: gw2-game-observe [--no-capture] --params BLOB136 --private EXPONENT64 --log DIR [--listen LOOPBACK:PORT] [--reply-frame FILE] [--auth-reply-body FILE [--auth-character-record BODY [--auth-play-route LOOPBACK:PORT]] | --mission-preamble [--mission-map-key KEY (= summary +1 Word of the auth character record) [--mission-population RECORD --mission-spawn X,Y,Z]]]",
                ));
            }
        }
    }
    if !bind_allowed(&listen, no_capture) {
        return Err(invalid(
            "non-loopback listen requires --no-capture and a specific address",
        ));
    }
    if preamble == Preamble::Mission && (auth_body.is_some() || character.is_some()) {
        return Err(invalid(
            "mission connections cannot use auth reply fixtures",
        ));
    }
    if mission_map_key.is_some() && (preamble != Preamble::Mission || reply.is_some()) {
        return Err(invalid(
            "--mission-map-key requires --mission-preamble and excludes --reply-frame",
        ));
    }
    if population_record.is_some() != spawn.is_some()
        || (population_record.is_some() && mission_map_key.is_none())
    {
        return Err(invalid(
            "--mission-population and --mission-spawn go together and require --mission-map-key",
        ));
    }
    let population = match (&population_record, spawn) {
        (Some(record), Some(spawn)) => Some(mission::PopulationFixture::new(record, spawn)?),
        _ => None,
    };
    if let Some(key) = mission_map_key {
        // Validate before binding; each connection builds its own phase.
        let phase = mission::MissionPhase::new(key)?;
        if let Some(fixture) = &population {
            phase.with_population(fixture.clone())?;
        }
    }
    if character.is_some() && auth_body.is_none() {
        return Err(invalid(
            "character record fixture requires --auth-reply-body",
        ));
    }
    if play_address.is_some() && character.is_none() {
        return Err(invalid(
            "play-route fixture requires --auth-character-record",
        ));
    }
    // The character value server 8 returns is the record's own leading Guid.
    let play = match (play_address, &character) {
        (Some(address), Some(record)) if record.len() >= 16 => Some(auth::PlayRoute::new(
            address,
            gw2_proto::Guid(record[..16].try_into().unwrap()),
        )?),
        _ => None,
    };
    let auth_body = auth_body
        .map(|body| auth::ReplyFixture::new(body, character))
        .transpose()?;
    let params = read_exact_file(&params.ok_or_else(|| invalid("missing --params"))?, 136)?;
    let params = ParamBlob::parse(&params, ByteOrder::Little)
        .ok_or_else(|| invalid("invalid parameters"))?;
    let private = read_exact_file(&private.ok_or_else(|| invalid("missing --private"))?, 64)?;
    let exponent = BigUint::from_bytes_le(&private);
    let one = BigUint::from(1u32);
    if params.version != 1
        || params.p.bits() != 512
        || params.g != BigUint::from(4u32)
        || (&params.p & &one) != one
        || exponent <= one
        || exponent >= &params.p - &one
    {
        return Err(invalid("invalid local DH identity"));
    }
    let key = ServerKey::from_private(params.p, params.g, exponent);
    if key.public() != params.server_public {
        return Err(invalid("public/private identity mismatch"));
    }
    let key = Arc::new(key);
    let logs = logs.ok_or_else(|| invalid("missing --log"))?;
    if !no_capture {
        fs::create_dir_all(&logs)?;
    }
    let listener = TcpListener::bind(listen)?;
    eprintln!("Game observer listening on {}", listener.local_addr()?);
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    for (index, stream) in listener.incoming().enumerate() {
        let stream = stream?;
        let key = Arc::clone(&key);
        let reply = reply.clone();
        let auth_body = auth_body.clone();
        let population = population.clone();
        let log = logs.join(format!("{epoch}-{index}.decrypted.bin"));
        std::thread::spawn(move || {
            let fixtures = Fixtures {
                reply: reply.as_ref(),
                auth_body,
                play,
                mission_map_key,
                population,
                no_capture,
            };
            if let Err(e) = observe(stream, &key, &log, fixtures, preamble) {
                eprintln!("Game observation ended: {e}");
            }
        });
    }
    Ok(())
}
