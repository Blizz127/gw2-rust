//! Offline build-209026 game-handshake observer. See docs/re/209026-game-handshake.md.
//! Uses an explicitly supplied local DH identity; captures decrypted bytes privately.
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
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
#[path = "game_observe/character_store.rs"]
mod character_store;
#[path = "game_observe/equipment.rs"]
mod equipment;
#[path = "game_observe/skills.rs"]
mod skills;
#[path = "game_observe/mission.rs"]
mod mission;
#[path = "game_observe/creation_scene.rs"]
mod creation_scene;
#[path = "game_observe/saved_population.rs"]
mod saved_population;
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
    saved_population: Option<saved_population::SavedPopulation>,
    creation: Option<(character_store::CharacterStore, SocketAddr)>,
    /// Diagnostic hold between client 420 and the population (zero: none).
    population_delay: Duration,
    /// Opt-in talkable test NPC (docs/re/209026-npc-talk-loop.md).
    talk_npc: Option<mission::TalkNpc>,
    /// Opt-in hostile test NPC: species and position.
    hostile_npc: Option<(u32, [f32; 3])>,
    /// Research: answer SkillBegin with server 34 + 0x78.
    skill_order_echo: bool,
    /// Research: hostile NPC casts this skill on each player SkillBegin.
    npc_cast: Option<u32>,
    /// Research: player skills aimed at the hostile NPC lower its health.
    damage_on_skill: Option<f32>,

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
        saved_population,
        creation,
        population_delay,
        talk_npc,
        hostile_npc,
        skill_order_echo,
        npc_cast,
        damage_on_skill,
        no_capture,
    } = fixtures;
    let mut mission = match (mission_map_key.map(mission::MissionPhase::new).transpose()?, population) {
        (Some(phase), Some(fixture)) => Some(phase.with_population(fixture)?.with_population_delay(population_delay)),
        (phase, _) => phase,
    };
    let mut auth = match (auth_body.map(auth::AuthProbe::new), play) {
        (Some(probe), Some(route)) => Some(probe.with_play_route(route)?),
        (probe, _) => probe,
    };
    let mut creation = creation.map(|(store, address)| creation_scene::CreationScene::new(store, address)).transpose()?;
    let play_enabled = auth.as_ref().is_some_and(auth::AuthProbe::play_enabled);
    let idle = read_timeout(play_enabled || mission.is_some() || creation.is_some());
    stream.set_read_timeout(Some(idle))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;
    // Sizes-only byte timeline (no contents), like the launcher relay's trace: it showed
    // the split key reply that broke the owner's login. Milliseconds since accept.
    let t0 = Instant::now();
    let ms = |t: Instant| t.elapsed().as_millis();
    let preamble = preamble.read(&mut stream)?;
    if let Some(saved) = saved_population {
        let fixture = saved.for_join(&preamble)?;
        mission = Some(mission.ok_or_else(|| invalid("saved population requires a mission phase"))?
            .with_population(fixture)?.with_population_delay(population_delay));
    }
    if let Some(npc) = talk_npc {
        mission = Some(mission.ok_or_else(|| invalid("talk NPC requires a mission phase"))?.with_talk_npc(npc));
    }
    if let Some((species, position)) = hostile_npc {
        mission = Some(
            mission
                .ok_or_else(|| invalid("hostile NPC requires a mission phase"))?
                .with_hostile_npc(species, position)?,
        );
    }
    if skill_order_echo {
        mission = Some(mission.ok_or_else(|| invalid("skill order echo requires a mission phase"))?.with_skill_order_echo());
    }
    if let Some(skill) = npc_cast {
        mission = Some(mission.ok_or_else(|| invalid("NPC cast requires a mission phase"))?.with_npc_cast(skill));
    }
    if let Some(amount) = damage_on_skill {
        mission = Some(mission.ok_or_else(|| invalid("damage requires a mission phase"))?.with_damage_on_skill(amount));
    }
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
    // The key reply and any immediate frames go out in ONE write on a NODELAY socket. The
    // client rejects a key reply that arrives split across reads (error 58:11:5:535, then
    // it closes and retries): written as 2 B + 20 B, Nagle held the second part back one
    // round trip on the owner's tailnet path (154 ms), and the client had already given up.
    stream.set_nodelay(true)?;
    let mut first = BytesMut::new();
    first.extend_from_slice(&[1, 22]);
    first.extend_from_slice(&block);
    if let Some(reply) = reply {
        codec.encode(reply.clone(), &mut first)?;
    }
    if let Some(mission) = &mission {
        codec.encode(mission.opening(), &mut first)?;
    }
    if let Some(creation) = &creation {
        codec.encode(creation.opening(), &mut first)?;
    }
    eprintln!("net +{}ms: up {} B (preamble + key record)", ms(t0), preamble.len() + len);
    stream.write_all(&first)?;
    eprintln!("net +{}ms: down {} B (key reply + first frames, one write)", ms(t0), first.len());
    eprintln!("Game key reply sent; client public length {}", public.len());
    if let Some(reply) = reply {
        eprintln!("Explicit game reply sent: {} plaintext bytes", reply.len());
    }
    if mission.is_some() {
        eprintln!("Mission phase sent server 1063 MapData and 1060 LoadMap");
    }
    if creation.is_some() {
        eprintln!("Creation scene sent server 1049 readiness");
    }
    let mut chunk = [0u8; 8192];
    let mut total = 0;
    // The 1 MiB cap bounds capture files; without capture the session is
    // bounded only by the read timeout.
    while no_capture || total < CAPTURE_LIMIT {
        // A held population shortens the read so it goes out on time (see --mission-population-delay-ms).
        let due = mission.as_ref().and_then(mission::MissionPhase::due_at);
        let wait = due.map_or(idle, |due| due.saturating_duration_since(Instant::now()).max(Duration::from_millis(1)));
        stream.set_read_timeout(Some(wait.min(idle)))?;
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
                if let Some(mission) = &mut mission
                    && let Some(held) = mission.take_due(Instant::now())
                {
                    let mut wire = BytesMut::new();
                    codec.encode(held, &mut wire)?;
                    stream.write_all(&wire)?;
                    eprintln!("net +{}ms: down {} B", ms(t0), wire.len());
                    continue;
                }
                if due.is_some() {
                    continue;
                }
                eprintln!("net +{}ms: read timeout", ms(t0));
                break;
            }
            Err(e) => {
                eprintln!("net +{}ms: read error {:?}", ms(t0), e.kind());
                return Err(e);
            }
        };
        if n == 0 {
            eprintln!("net +{}ms: client EOF", ms(t0));
            break;
        }
        eprintln!("net +{}ms: up {n} B", ms(t0));
        let mut pending = BytesMut::from(&chunk[..n]);
        if let Some(decoded) = codec.decode(&mut pending)? {
            capture.write_all(&decoded)?;
            capture.flush()?;
            total += decoded.len();
            eprintln!("Decrypted game bytes: {} (total {total})", decoded.len());
            // Everything owed for this read goes out in one write (see the key reply above).
            let mut wire = BytesMut::new();
            if let Some(auth) = &mut auth
                && let Some(reply) = auth.push(&decoded)?
            {
                codec.encode(reply, &mut wire)?;
                eprintln!("Explicit correlated auth fixture sent");
            }
            if let Some(mission) = &mut mission {
                for seen in mission.push(&decoded) {
                    if let Some(reply) = mission.respond(&seen)? {
                        codec.encode(reply, &mut wire)?;
                    }
                }
                if let Some(held) = mission.take_due(Instant::now()) {
                    codec.encode(held, &mut wire)?;
                }
            }
            if let Some(creation) = &mut creation
                && let Some(reply) = creation.push(&decoded)?
            {
                codec.encode(reply, &mut wire)?;
            }
            if !wire.is_empty() {
                stream.write_all(&wire)?;
                eprintln!("net +{}ms: down {} B", ms(t0), wire.len());
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

fn parse_csv<T: std::str::FromStr, const N: usize>(text: &str, message: &str) -> io::Result<[T; N]> {
    text.split(',')
        .map(str::parse)
        .collect::<Result<Vec<T>, _>>()
        .map_err(|_| invalid(message))?
        .try_into()
        .map_err(|_| invalid(message))
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
    let mut agreement_state: Option<std::path::PathBuf> = None;
    let mut spawn: Option<[f32; 3]> = None;
    let mut mesmer_values: Option<[u8; 2]> = None;
    let mut reward_tracks: Option<[u32; 4]> = None;
    let mut quest: Option<(u32, u8, u32)> = None;
    let mut equipment_frames: Option<Vec<u8>> = None;
    let mut skill_frames: Option<Vec<u8>> = None;
    let mut initial_health: Option<f32> = None;
    let mut talk_npc_spec: Option<(u32, [f32; 3])> = None;
    let mut talk_npc_speaker: Option<String> = None;
    let mut hostile_npc: Option<(u32, [f32; 3])> = None;
    let mut skill_order_echo = false;
    let mut npc_cast: Option<u32> = None;
    let mut damage_on_skill: Option<f32> = None;
    let mut store_path: Option<PathBuf> = None;
    let mut creation_destination: Option<SocketAddr> = None;
    let mut creation_route: Option<SocketAddr> = None;
    let mut population_delay = Duration::ZERO;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--character-store" => {
                if store_path.is_some() { return Err(invalid("duplicate --character-store")); }
                store_path = Some(PathBuf::from(args.next().ok_or_else(|| invalid("missing character store directory"))?));
            }
            "--creation-scene" | "--auth-create-route" => {
                let target = if arg == "--creation-scene" { &mut creation_destination } else { &mut creation_route };
                if target.is_some() { return Err(invalid("duplicate creation route option")); }
                let address: SocketAddr = args.next().ok_or_else(|| invalid("missing creation route address"))?
                    .parse().map_err(|_| invalid("invalid creation route address"))?;
                if !address.ip().is_loopback() { return Err(invalid("creation route address must be loopback")); }
                *target = Some(address);
            }
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
            "--auth-agreement-state" => {
                agreement_state = Some(args.next().ok_or_else(|| invalid("missing agreement state file"))?.into());
            }
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
            "--mission-population-delay-ms" => {
                let ms = args
                    .next()
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|ms| *ms <= 60_000)
                    .ok_or_else(|| invalid("invalid population delay (0..=60000 ms)"))?;
                population_delay = Duration::from_millis(ms);
            }
            "--mission-mesmer-values" => {
                if mesmer_values.is_some() {
                    return Err(invalid("duplicate --mission-mesmer-values"));
                }
                mesmer_values = Some(parse_csv(
                    &args.next().ok_or_else(|| invalid("missing Mesmer FIRST,SECOND"))?,
                    "Mesmer state needs exactly two bytes: FIRST,SECOND",
                )?);
            }
            "--mission-reward-tracks" => {
                if reward_tracks.is_some() {
                    return Err(invalid("duplicate --mission-reward-tracks"));
                }
                reward_tracks = Some(parse_csv(
                    &args.next().ok_or_else(|| invalid("missing reward tracks A,B,C,D"))?,
                    "reward tracks need exactly four unsigned Dwords: A,B,C,D",
                )?);
            }
            "--mission-quest" => {
                if quest.is_some() {
                    return Err(invalid("duplicate --mission-quest"));
                }
                let [key, flags, goal]: [u32; 3] = parse_csv(
                    &args.next().ok_or_else(|| invalid("missing quest KEY,FLAGS,GOAL"))?,
                    "quest needs exactly three unsigned values: KEY,FLAGS,GOAL",
                )?;
                let flags = u8::try_from(flags).map_err(|_| invalid("quest FLAGS must fit a byte"))?;
                if flags & 1 != 0 {
                    return Err(invalid("initial quest goal requires completion flag bit 0 clear"));
                }
                quest = Some((key, flags, goal));
            }
            "--mission-equipment" => {
                if equipment_frames.is_some() {
                    return Err(invalid("duplicate --mission-equipment"));
                }
                let path = args.next().ok_or_else(|| invalid("missing initial equipment frame file"))?;
                let mut frames = Vec::new();
                File::open(path)?.take((equipment::EQUIPMENT_BYTES_LIMIT + 1) as u64).read_to_end(&mut frames)?;
                equipment_frames = Some(frames);
            }
            "--mission-skills" => {
                if skill_frames.is_some() {
                    return Err(invalid("duplicate --mission-skills"));
                }
                let path = args.next().ok_or_else(|| invalid("missing initial skill frame file"))?;
                let mut frames = Vec::new();
                File::open(path)?.take((skills::SKILL_BYTES_LIMIT + 1) as u64).read_to_end(&mut frames)?;
                skill_frames = Some(frames);
            }
            "--mission-health" => {
                if initial_health.is_some() {
                    return Err(invalid("duplicate --mission-health"));
                }
                let text = args.next().ok_or_else(|| invalid("missing initial full health"))?;
                let value: f32 = text.parse().map_err(|_| invalid("invalid initial full health"))?;
                if !value.is_finite() || value <= 0.0 {
                    return Err(invalid("initial full health must be finite and positive"));
                }
                initial_health = Some(value);
            }
            "--mission-talk-npc" => {
                if talk_npc_spec.is_some() {
                    return Err(invalid("duplicate --mission-talk-npc"));
                }
                let text = args.next().ok_or_else(|| invalid("missing talk NPC SPECIES,X,Y,Z"))?;
                let parts: Vec<&str> = text.split(',').collect();
                let [species, x, y, z] = parts[..] else {
                    return Err(invalid("talk NPC needs exactly SPECIES,X,Y,Z"));
                };
                let species: u32 = species.parse().map_err(|_| invalid("invalid talk NPC species key"))?;
                let position = [x, y, z]
                    .map(|v| v.parse::<f32>().map_err(|_| invalid("invalid talk NPC position")));
                let [x, y, z] = position;
                talk_npc_spec = Some((species, [x?, y?, z?]));
            }
            "--mission-skill-order-echo" => skill_order_echo = true,
            "--mission-damage-on-skill" => {
                let text = args.next().ok_or_else(|| invalid("missing damage amount"))?;
                let value: f32 = text.parse().map_err(|_| invalid("invalid damage amount"))?;
                if !value.is_finite() || value <= 0.0 {
                    return Err(invalid("damage amount must be finite and positive"));
                }
                damage_on_skill = Some(value);
            }
            "--mission-npc-cast" => {
                let text = args.next().ok_or_else(|| invalid("missing NPC cast skill key"))?;
                npc_cast = Some(text.parse().map_err(|_| invalid("invalid NPC cast skill key"))?);
            }
            "--mission-hostile-npc" => {
                if hostile_npc.is_some() {
                    return Err(invalid("duplicate --mission-hostile-npc"));
                }
                let text = args.next().ok_or_else(|| invalid("missing hostile NPC SPECIES,X,Y,Z"))?;
                let parts: Vec<&str> = text.split(',').collect();
                let [species, x, y, z] = parts[..] else {
                    return Err(invalid("hostile NPC needs exactly SPECIES,X,Y,Z"));
                };
                let species: u32 = species.parse().map_err(|_| invalid("invalid hostile NPC species key"))?;
                let [x, y, z] = [x, y, z].map(|v| v.parse::<f32>().map_err(|_| invalid("invalid hostile NPC position")));
                hostile_npc = Some((species, [x?, y?, z?]));
            }
            "--mission-talk-npc-speaker" => {
                if talk_npc_speaker.is_some() {
                    return Err(invalid("duplicate --mission-talk-npc-speaker"));
                }
                talk_npc_speaker = Some(args.next().ok_or_else(|| invalid("missing talk NPC speaker"))?);
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
                    "usage: gw2-game-observe [--no-capture] [--character-store DIR] [--auth-create-route LOOPBACK:PORT] [--creation-scene WORLD_LOOPBACK:PORT] --params BLOB136 --private EXPONENT64 --log DIR [--listen LOOPBACK:PORT] [--reply-frame FILE] [--auth-reply-body FILE [--auth-agreement-state FILE] [--auth-character-record BODY [--auth-play-route LOOPBACK:PORT]] | --mission-preamble [--mission-map-key KEY (= summary +1 Word of the auth character record) [--mission-population RECORD --mission-spawn X,Y,Z [--mission-population-delay-ms MS] [--mission-mesmer-values FIRST,SECOND --mission-reward-tracks A,B,C,D] [--mission-equipment FILE] [--mission-skills FILE] [--mission-health VALUE] [--mission-quest KEY,FLAGS,GOAL] [--mission-talk-npc SPECIES,X,Y,Z [--mission-talk-npc-speaker NAME]] [--mission-hostile-npc SPECIES,X,Y,Z] [--mission-skill-order-echo] [--mission-npc-cast SKILL] [--mission-damage-on-skill AMOUNT]]]]",
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
    if creation_destination.is_some() && (preamble != Preamble::Mission || store_path.is_none()
        || mission_map_key.is_some() || reply.is_some() || auth_body.is_some()) {
        return Err(invalid("--creation-scene requires --mission-preamble and --character-store, without map/auth/reply fixtures"));
    }
    if store_path.is_some() && (!no_capture || (auth_body.is_none() && mission_map_key.is_none() && creation_destination.is_none())) {
        return Err(invalid("--character-store requires --no-capture and an auth, mission, or creation role"));
    }
    if store_path.is_some() && (character.is_some() || population_record.is_some()) {
        return Err(invalid("saved characters and static character/population fixtures are mutually exclusive"));
    }
    if creation_route.is_some() && (auth_body.is_none() || play_address.is_none() || store_path.is_none()) {
        return Err(invalid("--auth-create-route requires auth body, play route and character store"));
    }
    let saved_population_mode = store_path.is_some() && mission_map_key.is_some();
    let has_population = population_record.is_some() || saved_population_mode;
    if has_population != spawn.is_some()
        || (has_population && mission_map_key.is_none())
    {
        return Err(invalid(
            "--mission-population and --mission-spawn go together and require --mission-map-key",
        ));
    }
    if agreement_state.is_some() && auth_body.is_none() {
        return Err(invalid("--auth-agreement-state requires --auth-reply-body"));
    }
    if !population_delay.is_zero() && !has_population {
        return Err(invalid("--mission-population-delay-ms requires --mission-population"));
    }
    if mesmer_values.is_some() != reward_tracks.is_some()
        || (mesmer_values.is_some() && !has_population)
    {
        return Err(invalid("--mission-mesmer-values and --mission-reward-tracks go together and require --mission-population"));
    }
    if quest.is_some() && !has_population {
        return Err(invalid("--mission-quest requires --mission-population"));
    }
    if equipment_frames.is_some() && !has_population {
        return Err(invalid("--mission-equipment requires --mission-population"));
    }
    if skill_frames.is_some() && !has_population {
        return Err(invalid("--mission-skills requires --mission-population"));
    }
    if initial_health.is_some() && !has_population {
        return Err(invalid("--mission-health requires --mission-population"));
    }
    if damage_on_skill.is_some() && hostile_npc.is_none() {
        return Err(invalid("--mission-damage-on-skill requires --mission-hostile-npc"));
    }
    if npc_cast.is_some() && hostile_npc.is_none() {
        return Err(invalid("--mission-npc-cast requires --mission-hostile-npc"));
    }
    if skill_order_echo && !has_population {
        return Err(invalid("--mission-skill-order-echo requires a population"));
    }
    if hostile_npc.is_some() && !has_population {
        return Err(invalid("--mission-hostile-npc requires a population"));
    }
    if (talk_npc_spec.is_some() && !has_population) || (talk_npc_speaker.is_some() && talk_npc_spec.is_none()) {
        return Err(invalid("--mission-talk-npc requires a population; --mission-talk-npc-speaker requires --mission-talk-npc"));
    }
    let talk_npc = talk_npc_spec
        .map(|(species, position)| {
            mission::TalkNpc::new(species, position, talk_npc_speaker.unwrap_or_else(|| "Talk test".to_owned()))
        })
        .transpose()?;
    if saved_population_mode && (mission_map_key != Some(379) || mesmer_values.is_none()) {
        return Err(invalid("saved population currently requires Norn map379 and explicit Mesmer state/tracks"));
    }
    let store = store_path.map(character_store::CharacterStore::open).transpose()?;
    let saved_population = if saved_population_mode {
        let profile = saved_population::SavedPopulation {
            store: store.clone().unwrap(), spawn: spawn.unwrap(),
            profession: gw2_proto::population::PopulationProfession209026::Mesmer { values: mesmer_values.unwrap() },
            reward_tracks: reward_tracks.unwrap(), quest,
            equipment: equipment_frames.as_deref().map(Bytes::copy_from_slice),
            skills: skill_frames.as_deref().map(Bytes::copy_from_slice), initial_health,
        };
        profile.validate()?;
        Some(profile)
    } else { None };
    let creation = creation_destination.map(|address| (store.clone().unwrap(), address));
    let population = match (&population_record, spawn) {
        (Some(record), Some(spawn)) => Some(match (mesmer_values, reward_tracks) {
            (Some(values), Some(tracks)) => mission::PopulationFixture::with_profession(
                record, spawn, gw2_proto::population::PopulationProfession209026::Mesmer { values }, tracks,
            )?,
            _ => mission::PopulationFixture::new(record, spawn)?,
        }),
        _ => None,
    };
    let population = match (population, quest) {
        (Some(fixture), Some((key, flags, goal))) => Some(fixture.with_quest(key, flags, goal)?),
        (fixture, _) => fixture,
    };
    let population = match (population, equipment_frames) {
        (Some(fixture), Some(frames)) => Some(fixture.with_equipment(&frames)?),
        (fixture, _) => fixture,
    };
    let population = match (population, skill_frames) {
        (Some(fixture), Some(frames)) => Some(fixture.with_skills(&frames)?),
        (fixture, _) => fixture,
    };
    let population = match (population, initial_health) {
        (Some(fixture), Some(value)) => Some(fixture.with_initial_health(value)?),
        (fixture, _) => fixture,
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
    if play_address.is_some() && character.is_none() && store.is_none() {
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
        (Some(address), None) if store.is_some() => Some(auth::PlayRoute::new(address, gw2_proto::Guid([0; 16]))?),
        _ => None,
    };
    let play = match (play, creation_route) {
        (Some(route), Some(address)) => Some(route.with_creation_address(address)?),
        (route, _) => route,
    };
    let auth_body = auth_body
        .map(|body| {
            let fixture = auth::ReplyFixture::new(body, character).and_then(auth::ReplyFixture::with_home_world)?;
            let fixture = match &store {
                Some(store) => fixture.with_character_store(store.clone())?,
                None => fixture,
            };
            Ok::<_, io::Error>(match agreement_state {
                Some(path) => fixture.with_agreement(auth::AgreementState::new(path)),
                None => fixture,
            })
        })
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
        let saved_population = saved_population.clone();
        let creation = creation.clone();
        let talk_npc = talk_npc.clone();
        let log = logs.join(format!("{epoch}-{index}.decrypted.bin"));
        std::thread::spawn(move || {
            let fixtures = Fixtures {
                reply: reply.as_ref(),
                auth_body,
                play,
                mission_map_key,
                population,
                saved_population,
                creation,
                population_delay,
                talk_npc,
                hostile_npc,
                skill_order_echo,
                npc_cast,
                damage_on_skill,
                no_capture,
            };
            if let Err(e) = observe(stream, &key, &log, fixtures, preamble) {
                eprintln!("Game observation ended: {e}");
            }
        });
    }
    Ok(())
}
