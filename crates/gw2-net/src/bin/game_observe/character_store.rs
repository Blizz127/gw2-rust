//! Explicit single-account sandbox persistence, not an account/auth service.
//! The original client-415 frame is the durable source of appearance, answers
//! and colors. Derived summary/record fields not established by the source use
//! the same explicitly research-only zeros as `gw2-character-fixture`.

use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use bytes::{Bytes, BytesMut};
use gw2_proto::{
    Encode, Guid,
    character_creation::{CreationRequest209026, MAX_FRAME_SIZE},
    character_summary::{SummaryV11, V11_SIZE},
};
#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};

/// Local storage limits, not retail account-slot rules.
pub const MAX_CHARACTERS: usize = 64;
const MAX_DIRECTORY_ENTRIES: usize = 129;
const MAGIC: &[u8; 8] = b"GW2CHR01";
const BUILD: u32 = 209026;
const HEADER_SIZE: usize = 30;
const MAX_RECORD_SIZE: usize = HEADER_SIZE + MAX_FRAME_SIZE;

#[derive(Clone)]
pub struct SavedCharacter {
    guid: Guid,
    name: String,
    request: Bytes,
    summary: [u8; V11_SIZE],
}

impl SavedCharacter {
    pub fn guid(&self) -> Guid {
        self.guid
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Exact complete 415 frame, including all 48 appearance bytes and all
    /// supplied answer/color values. No biography choices are synthesized.
    pub fn request(&self) -> &[u8] {
        &self.request
    }
    pub fn summary(&self) -> &[u8; V11_SIZE] {
        &self.summary
    }

    /// Server-17 record body, excluding message/request IDs. Unknown record
    /// fields and preview equipment are research zeros, as in the fixture tool.
    pub fn auth_body(&self) -> Bytes {
        let mut body = BytesMut::new();
        self.guid.encode(&mut body);
        0u32.encode(&mut body);
        0u64.encode(&mut body);
        0u64.encode(&mut body);
        for unit in self.name.encode_utf16().chain(std::iter::once(0)) {
            unit.encode(&mut body);
        }
        (V11_SIZE as u16).encode(&mut body);
        body.extend_from_slice(&self.summary);
        body.freeze()
    }
}

// No Debug: names and raw character data should not enter routine observer logs.
#[derive(Clone)]
pub struct CharacterStore(Arc<StoreInner>);

struct StoreInner {
    directory: PathBuf,
    // flock-style locks can share ownership across cloned file handles. The
    // mutex is necessary for threads using clones of this store; OS locking
    // additionally covers independent opens and independent processes.
    lock: Mutex<File>,
}

struct StoreLock<'a>(MutexGuard<'a, File>);
impl Drop for StoreLock<'_> {
    fn drop(&mut self) {
        let _ = File::unlock(&self.0);
    }
}

impl CharacterStore {
    /// Open/create a private directory whose parent already exists. Existing
    /// permissions/data are checked, not repaired or overwritten. Corrupt
    /// committed records make opening fail and remain on disk for inspection.
    pub fn open(directory: PathBuf) -> io::Result<Self> {
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        builder.mode(0o700);
        match builder.create(&directory) {
            Ok(()) => sync_directory(
                directory
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or(Path::new(".")),
            )?,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        let metadata = fs::symlink_metadata(&directory)?;
        if !metadata.is_dir() {
            return Err(corrupt("character store must be a directory, not a link"));
        }
        check_private(&metadata)?;
        let directory = fs::canonicalize(directory)?;
        let lock_path = directory.join(".lock");
        let lock = match private_new(&lock_path) {
            Ok(file) => {
                file.sync_all()?;
                sync_directory(&directory)?;
                file
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                open_private(&lock_path, true)?
            }
            Err(error) => return Err(error),
        };
        if lock.metadata()?.len() != 0 {
            return Err(corrupt("invalid character-store lock file"));
        }
        let store = Self(Arc::new(StoreInner {
            directory,
            lock: Mutex::new(lock),
        }));
        store.list()?;
        Ok(store)
    }

    fn lock(&self) -> io::Result<StoreLock<'_>> {
        let file = self
            .0
            .lock
            .lock()
            .map_err(|_| io::Error::other("character-store mutex poisoned"))?;
        File::lock(&file)?;
        Ok(StoreLock(file))
    }

    pub fn list(&self) -> io::Result<Vec<SavedCharacter>> {
        let _lock = self.lock()?;
        self.load_locked()
    }

    /// Local uniqueness uses Unicode lowercase without normalization. This is
    /// an explicit sandbox policy, not an inferred retail name policy.
    pub fn find_name(&self, name: &str) -> io::Result<Option<SavedCharacter>> {
        let key = name.to_lowercase();
        Ok(self
            .list()?
            .into_iter()
            .find(|record| record.name.to_lowercase() == key))
    }

    pub fn find_guid(&self, guid: Guid) -> io::Result<Option<SavedCharacter>> {
        Ok(self.list()?.into_iter().find(|record| record.guid == guid))
    }

    pub fn create(&self, request: &[u8]) -> io::Result<SavedCharacter> {
        let (name, summary) = validate_request(request).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "unsupported or malformed sandbox creation request",
            )
        })?;
        let _lock = self.lock()?;
        let records = self.load_locked()?;
        let key = name.to_lowercase();
        if let Some(record) = records.iter().find(|r| r.name.to_lowercase() == key) {
            if record.request() == request {
                // Also complete a previous publish whose directory sync failed.
                sync_directory(&self.0.directory)?;
                return Ok(record.clone());
            }
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "sandbox character name already exists with a different request",
            ));
        }
        if records.len() >= MAX_CHARACTERS {
            return Err(io::Error::new(
                io::ErrorKind::StorageFull,
                "sandbox character limit reached",
            ));
        }
        for _ in 0..16 {
            let guid = random_guid()?;
            let destination = self.0.directory.join(format!("{}.char", guid_hex(guid)));
            let pending = self
                .0
                .directory
                .join(format!(".pending-{}", guid_hex(guid)));
            if destination.try_exists()? || pending.try_exists()? {
                continue;
            }
            let mut file = match private_new(&pending) {
                Ok(file) => file,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            };
            let _cleanup = PendingFile(pending.clone());
            file.write_all(MAGIC)?;
            file.write_all(&BUILD.to_le_bytes())?;
            file.write_all(&guid.0)?;
            file.write_all(&(request.len() as u16).to_le_bytes())?;
            file.write_all(request)?;
            file.sync_all()?;
            // Atomic publication without rename's overwrite semantics. Both
            // names are in this directory and thus on the same filesystem.
            match fs::hard_link(&pending, &destination) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
            fs::remove_file(&pending)?;
            sync_directory(&self.0.directory)?;
            return Ok(SavedCharacter {
                guid,
                name,
                request: Bytes::copy_from_slice(request),
                summary,
            });
        }
        Err(io::Error::other(
            "could not allocate a unique character record identity",
        ))
    }

    fn load_locked(&self) -> io::Result<Vec<SavedCharacter>> {
        let mut paths = Vec::new();
        for (index, entry) in fs::read_dir(&self.0.directory)?.enumerate() {
            if index >= MAX_DIRECTORY_ENTRIES {
                return Err(corrupt("character-store directory entry limit exceeded"));
            }
            let entry = entry?;
            let filename = entry.file_name();
            let filename = filename
                .to_str()
                .ok_or_else(|| corrupt("invalid character-store filename"))?;
            if filename == ".lock" {
                continue;
            }
            if !entry.file_type()?.is_file() {
                return Err(corrupt("character-store entry is not a regular file"));
            }
            check_private(&fs::symlink_metadata(entry.path())?)?;
            // Uncommitted crash leftovers are bounded, ignored and preserved.
            // They can also be a second link to an already committed record.
            if filename
                .strip_prefix(".pending-")
                .is_some_and(valid_guid_hex)
            {
                continue;
            }
            let hex = filename
                .strip_suffix(".char")
                .filter(|hex| valid_guid_hex(hex))
                .ok_or_else(|| corrupt("unrecognized character-store entry"))?;
            paths.push((entry.path(), hex.to_owned()));
            if paths.len() > MAX_CHARACTERS {
                return Err(corrupt("character-store record limit exceeded"));
            }
        }
        paths.sort_by(|a, b| a.1.cmp(&b.1));
        let mut names = HashSet::new();
        let mut records = Vec::with_capacity(paths.len());
        for (path, hex) in paths {
            let file = open_private(&path, false)?;
            if file.metadata()?.len() > MAX_RECORD_SIZE as u64 {
                return Err(corrupt("oversized character-store record"));
            }
            let mut bytes = Vec::new();
            file.take((MAX_RECORD_SIZE + 1) as u64)
                .read_to_end(&mut bytes)?;
            if bytes.len() < HEADER_SIZE
                || bytes.len() > MAX_RECORD_SIZE
                || &bytes[..8] != MAGIC
                || bytes[8..12] != BUILD.to_le_bytes()
            {
                return Err(corrupt("invalid character-store record header"));
            }
            let guid = Guid(bytes[12..28].try_into().unwrap());
            let length = u16::from_le_bytes(bytes[28..30].try_into().unwrap()) as usize;
            if guid.0 == [0; 16]
                || guid_hex(guid) != hex
                || length > MAX_FRAME_SIZE
                || bytes.len() != HEADER_SIZE + length
            {
                return Err(corrupt("invalid character-store record framing"));
            }
            let request = &bytes[HEADER_SIZE..];
            let (name, summary) = validate_request(request)?;
            if !names.insert(name.to_lowercase()) {
                return Err(corrupt("duplicate character-store name"));
            }
            records.push(SavedCharacter {
                guid,
                name,
                request: Bytes::copy_from_slice(request),
                summary,
            });
        }
        Ok(records)
    }
}

fn validate_request(bytes: &[u8]) -> io::Result<(String, [u8; V11_SIZE])> {
    let request = CreationRequest209026::parse(bytes)?;
    let name = String::from_utf16(request.name_units())
        .map_err(|_| corrupt("invalid UTF-16 character name"))?;
    if !request.passes_context_scalar_checks()
        || request.appearance()[27] != 3
        || request.appearance()[28] > 15
        || request.profession() != 7
        || request.level() != 1
        || !matches!(request.flags(), 0 | 8)
        || name.is_empty()
        || name.chars().any(char::is_control)
    {
        return Err(corrupt("unsupported sandbox character scalars or name"));
    }
    super::equipment::mesmer_mask_choice(request.answers())
        .map_err(|_| corrupt("saved Mesmer requires exactly one profession biography answer"))?;
    // Armor delivery uses Words for all six dye rows. This is a sandbox
    // representability requirement, not validation against a retail palette.
    if request
        .item_colors()
        .iter()
        .flatten()
        .any(|&color| color > u32::from(u16::MAX))
    {
        return Err(corrupt(
            "saved creation color cannot be represented by armor delivery",
        ));
    }
    // Version 11, map 379, level 1, Mesmer 7, first 45 appearance bytes.
    // Remaining summary bytes (including preview equipment) are research zeros.
    let mut summary = [0; V11_SIZE];
    summary[0] = 11;
    summary[1..3].copy_from_slice(&379u16.to_le_bytes());
    summary[4] = 1;
    summary[5] = 7;
    summary[6..51].copy_from_slice(&request.appearance()[..45]);
    SummaryV11::parse(&summary)?;
    Ok((name, summary))
}

fn corrupt(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn random_guid() -> io::Result<Guid> {
    for _ in 0..16 {
        let mut bytes = [0; 16];
        getrandom::fill(&mut bytes).map_err(io::Error::other)?;
        if bytes != [0; 16] {
            return Ok(Guid(bytes));
        }
    }
    Err(io::Error::other(
        "random generator returned only zero identities",
    ))
}
fn guid_hex(guid: Guid) -> String {
    guid.0.iter().map(|b| format!("{b:02x}")).collect()
}
fn valid_guid_hex(text: &str) -> bool {
    text.len() == 32
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn check_private(metadata: &fs::Metadata) -> io::Result<()> {
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "character-store permissions must be private",
        ));
    }
    Ok(())
}
fn private_new(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    options.open(path)
}
fn open_private(path: &Path, write: bool) -> io::Result<File> {
    let before = fs::symlink_metadata(path)?;
    if !before.is_file() {
        return Err(corrupt("character-store file must be regular, not a link"));
    }
    check_private(&before)?;
    let file = OpenOptions::new().read(true).write(write).open(path)?;
    let after = file.metadata()?;
    if !after.is_file() {
        return Err(corrupt("character-store file changed type"));
    }
    check_private(&after)?;
    #[cfg(unix)]
    if before.dev() != after.dev() || before.ino() != after.ino() {
        return Err(corrupt("character-store file changed while opening"));
    }
    Ok(file)
}
fn sync_directory(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}
struct PendingFile(PathBuf);
impl Drop for PendingFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// Synthetic framing/scalar fixture for observer tests; not validated biography
/// or cosmetic content. Production always retains the user's actual 415 frame.
#[cfg(test)]
pub(super) fn test_request(name: &str) -> Vec<u8> {
    let mut bytes = BytesMut::new();
    415u16.encode(&mut bytes);
    for unit in name.encode_utf16().chain(std::iter::once(0)) {
        unit.encode(&mut bytes);
    }
    let mut appearance = [0; 48];
    appearance[27..30].copy_from_slice(&[3, 9, 1]);
    appearance[45..].copy_from_slice(&[0xad, 0xbe, 0xef]);
    bytes.extend_from_slice(&appearance);
    7u8.encode(&mut bytes);
    1u8.encode(&mut bytes);
    8u32.encode(&mut bytes);
    bytes.extend_from_slice(&[3, 187, 1, 7]);
    for color in [0, 127, 128, 300, u32::from(u16::MAX)]
        .into_iter()
        .chain(std::iter::repeat_n(0, 19))
    {
        color.encode(&mut bytes);
    }
    bytes.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gw2_proto::Decode;
    use std::{
        process::{Command, Stdio},
        sync::Barrier,
        time::{Duration, Instant},
    };

    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "gw2-character-store-{}",
                guid_hex(random_guid().unwrap())
            ));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            builder.mode(0o700);
            builder.create(&path).unwrap();
            Self(path)
        }
        fn store_path(&self) -> PathBuf {
            self.0.join("store")
        }
        fn store(&self) -> CharacterStore {
            CharacterStore::open(self.store_path()).unwrap()
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn persists_all_request_bytes_and_reopens_with_stable_identity() {
        let temp = Temp::new();
        let request = test_request("Saved Ω");
        let first = temp.store().create(&request).unwrap();
        assert_ne!(first.guid(), Guid([0; 16]));
        let parsed = CreationRequest209026::parse(first.request()).unwrap();
        assert_eq!(&parsed.appearance()[45..], &[0xad, 0xbe, 0xef]);
        assert_eq!(parsed.answers(), &[187, 1, 7]);
        assert_eq!(parsed.item_colors()[0], [0, 127, 128, 300]);
        let reopened = temp.store();
        let record = reopened.find_name("SAVED Ω").unwrap().unwrap();
        assert_eq!(record.guid(), first.guid());
        assert_eq!(record.request(), request);
        assert_eq!(
            reopened.find_guid(first.guid()).unwrap().unwrap().name(),
            "Saved Ω"
        );
        let summary = SummaryV11::parse(record.summary()).unwrap();
        assert_eq!(
            (
                summary.map_definition_key(),
                summary.level(),
                summary.profession(),
                summary.race()
            ),
            (379, 1, 7, 3)
        );
        assert_eq!(summary.appearance(), &parsed.appearance()[..45]);
        assert!(record.summary()[51..].iter().all(|&b| b == 0));
        let body = record.auth_body();
        let mut input = body.as_ref();
        assert_eq!(Guid::decode(&mut input).unwrap(), first.guid());
        assert_eq!(u32::decode(&mut input).unwrap(), 0);
        assert_eq!(u64::decode(&mut input).unwrap(), 0);
        assert_eq!(u64::decode(&mut input).unwrap(), 0);
        for unit in "Saved Ω".encode_utf16().chain(std::iter::once(0)) {
            assert_eq!(u16::decode(&mut input).unwrap(), unit);
        }
        assert_eq!(u16::decode(&mut input).unwrap(), 159);
        assert_eq!(input, record.summary());
        let (_, auth_summary) = crate::auth::character_parts(&body).unwrap();
        assert_eq!(auth_summary, record.summary());
        #[cfg(unix)]
        {
            assert_eq!(
                fs::metadata(temp.store_path())
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o700
            );
            for file in fs::read_dir(temp.store_path()).unwrap() {
                assert_eq!(
                    file.unwrap().metadata().unwrap().permissions().mode() & 0o777,
                    0o600
                );
            }
        }
    }

    #[test]
    fn retries_are_exact_and_name_conflicts_never_overwrite() {
        let temp = Temp::new();
        let store = temp.store();
        let request = test_request("Aster");
        let first = store.create(&request).unwrap();
        assert_eq!(store.create(&request).unwrap().guid(), first.guid());
        let mut altered = request.clone();
        *altered.last_mut().unwrap() = 1;
        for conflict in [altered, test_request("ASTER")] {
            assert_eq!(
                store.create(&conflict).err().unwrap().kind(),
                io::ErrorKind::AlreadyExists
            );
        }
        store.create(&test_request("../Birch")).unwrap();
        assert_eq!(store.list().unwrap().len(), 2);
        assert_eq!(
            store.find_name("aster").unwrap().unwrap().request(),
            request
        );
        assert!(store.find_guid(Guid([0; 16])).unwrap().is_none());
        assert!(!temp.0.join("Birch").exists());
    }

    #[test]
    fn rejects_unsupported_requests_before_creating_any_record() {
        let temp = Temp::new();
        let store = temp.store();
        let good = test_request("Norn");
        let appearance = 2 + 2 * (4 + 1);
        let mut bad = vec![
            test_request(""),
            test_request("Line\nBreak"),
            test_request("A".repeat(20).as_str()),
        ];
        for (offset, value) in [
            (2, 0),
            (appearance + 27, 2),
            (appearance + 28, 16),
            (appearance + 29, 2),
            (appearance + 2, 255),
            (appearance + 48, 1),
            (appearance + 49, 2),
            (appearance + 50, 1),
        ] {
            let mut request = good.clone();
            request[offset] = value;
            bad.push(request);
        }
        let mut surrogate = good.clone();
        surrogate[2..4].copy_from_slice(&0xd800u16.to_le_bytes());
        bad.push(surrogate);
        let mut trailing = good.clone();
        trailing.push(0);
        bad.push(trailing);
        for request in bad {
            assert_eq!(
                store.create(&request).err().unwrap().kind(),
                io::ErrorKind::InvalidInput
            );
        }
        assert!(store.list().unwrap().is_empty());
        assert_eq!(fs::read_dir(temp.store_path()).unwrap().count(), 1);
    }

    #[test]
    fn biography_admission_requires_exactly_one_choice_without_rewriting_answers() {
        let temp = Temp::new();
        let store = temp.store();
        let name = "Biography";
        let count_offset = 2 + 2 * (name.encode_utf16().count() + 1) + 48 + 3;
        for answers in [&[][..], &[7, 1, 7], &[187, 187], &[187, 188], &[188, 189]] {
            let mut request = test_request(name);
            request.splice(
                count_offset..count_offset + 4,
                std::iter::once(answers.len() as u8).chain(answers.iter().copied()),
            );
            assert_eq!(
                store.create(&request).err().unwrap().kind(),
                io::ErrorKind::InvalidInput
            );
        }
        assert!(store.list().unwrap().is_empty());
        assert_eq!(fs::read_dir(temp.store_path()).unwrap().count(), 1);
        for choice in 187..=189 {
            let name = format!("Choice {choice}");
            let mut request = test_request(&name);
            let answer_offset = 2 + 2 * (name.encode_utf16().count() + 1) + 48 + 4;
            request[answer_offset] = choice;
            let saved = store.create(&request).unwrap();
            assert_eq!(saved.request(), request);
            assert_eq!(
                CreationRequest209026::parse(saved.request())
                    .unwrap()
                    .answers(),
                &[choice, 1, 7]
            );
        }
        assert_eq!(temp.store().list().unwrap().len(), 3);
    }

    #[test]
    fn creation_colors_require_word_representability_in_every_row() {
        let temp = Temp::new();
        let store = temp.store();
        let name = "Dye bounds";
        let color_offset = 2 + 2 * (name.encode_utf16().count() + 1) + 48 + 3 + 4;
        for invalid_index in 0..24 {
            let mut request = BytesMut::from(&test_request(name)[..color_offset]);
            for index in 0..24 {
                let color = if index == invalid_index {
                    65536u32
                } else {
                    65535
                };
                color.encode(&mut request);
            }
            assert_eq!(
                store.create(&request).err().unwrap().kind(),
                io::ErrorKind::InvalidInput
            );
        }
        assert!(store.list().unwrap().is_empty());
        let mut request = BytesMut::from(&test_request(name)[..color_offset]);
        for _ in 0..24 {
            65535u32.encode(&mut request);
        }
        let saved = store.create(&request).unwrap();
        assert_eq!(
            CreationRequest209026::parse(saved.request())
                .unwrap()
                .item_colors(),
            &[[65535; 4]; 6]
        );
        assert_eq!(
            temp.store()
                .find_guid(saved.guid())
                .unwrap()
                .unwrap()
                .request(),
            request
        );
    }

    #[test]
    fn loaded_record_with_missing_biography_is_rejected_and_preserved() {
        let temp = Temp::new();
        let store = temp.store();
        let name = "Biography";
        let saved = store.create(&test_request(name)).unwrap();
        let path = temp
            .store_path()
            .join(format!("{}.char", guid_hex(saved.guid())));
        let mut bytes = fs::read(&path).unwrap();
        let answer_offset = HEADER_SIZE + 2 + 2 * (name.encode_utf16().count() + 1) + 48 + 4;
        bytes[answer_offset] = 7;
        fs::write(&path, &bytes).unwrap();
        assert_eq!(
            store.list().err().unwrap().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(
            CharacterStore::open(temp.store_path())
                .err()
                .unwrap()
                .kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }

    #[test]
    fn scale_admission_matches_world_population_bounds() {
        let temp = Temp::new();
        let store = temp.store();
        for scale in [0, 15, 16, 254, 255] {
            let name = format!("Scale {scale}");
            let mut request = test_request(&name);
            let appearance = 2 + 2 * (name.encode_utf16().count() + 1);
            request[appearance + 28] = scale;
            if scale <= 15 {
                let saved = store.create(&request).unwrap();
                assert_eq!(saved.summary()[6 + 28], scale);
                super::super::mission::PopulationFixture::with_profession(
                    &saved.auth_body(),
                    [0.0; 3],
                    gw2_proto::population::PopulationProfession209026::Mesmer { values: [3, 0] },
                    [0; 4],
                )
                .unwrap();
            } else {
                assert_eq!(
                    store.create(&request).err().unwrap().kind(),
                    io::ErrorKind::InvalidInput
                );
                assert!(store.find_name(&name).unwrap().is_none());
            }
        }
        assert_eq!(store.list().unwrap().len(), 2);
    }

    #[test]
    fn malformed_committed_records_error_without_modifying_them() {
        for corruption in 0..5 {
            let temp = Temp::new();
            let store = temp.store();
            let saved = store.create(&test_request("Corruption probe")).unwrap();
            let path = temp
                .store_path()
                .join(format!("{}.char", guid_hex(saved.guid())));
            let mut bytes = fs::read(&path).unwrap();
            match corruption {
                0 => {
                    bytes.pop();
                }
                1 => bytes[0] ^= 1,
                2 => bytes[12..28].fill(0),
                3 => bytes.extend(std::iter::repeat_n(0, MAX_RECORD_SIZE)),
                _ => bytes[HEADER_SIZE] = 0,
            }
            fs::write(&path, &bytes).unwrap();
            assert!(store.list().is_err());
            assert!(CharacterStore::open(temp.store_path()).is_err());
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
    }

    #[test]
    fn pending_files_are_preserved_and_entries_and_record_counts_are_bounded() {
        let temp = Temp::new();
        let store = temp.store();
        let pending = temp
            .store_path()
            .join(format!(".pending-{}", guid_hex(Guid([1; 16]))));
        private_new(&pending)
            .unwrap()
            .write_all(b"interrupted write")
            .unwrap();
        assert!(store.list().unwrap().is_empty());
        assert_eq!(fs::read(&pending).unwrap(), b"interrupted write");
        let first_request = test_request("Slot 0");
        let first = store.create(&first_request).unwrap();
        for i in 1..MAX_CHARACTERS {
            store.create(&test_request(&format!("Slot {i}"))).unwrap();
        }
        assert_eq!(store.list().unwrap().len(), MAX_CHARACTERS);
        assert_eq!(store.create(&first_request).unwrap().guid(), first.guid());
        assert_eq!(
            store
                .create(&test_request("One too many"))
                .err()
                .unwrap()
                .kind(),
            io::ErrorKind::StorageFull
        );
        for i in 2..=MAX_DIRECTORY_ENTRIES {
            let mut bytes = [0; 16];
            bytes[..8].copy_from_slice(&(i as u64).to_le_bytes());
            private_new(
                &temp
                    .store_path()
                    .join(format!(".pending-{}", guid_hex(Guid(bytes)))),
            )
            .unwrap();
        }
        assert!(store.list().is_err());
        assert!(pending.exists());
    }

    #[test]
    fn clones_and_independent_opens_serialize_concurrent_retries() {
        let temp = Temp::new();
        let store = temp.store();
        let barrier = Arc::new(Barrier::new(8));
        let mut threads = Vec::new();
        for i in 0..8 {
            let handle = if i % 2 == 0 {
                store.clone()
            } else {
                temp.store()
            };
            let barrier = barrier.clone();
            threads.push(std::thread::spawn(move || {
                barrier.wait();
                handle.create(&test_request("Same request")).unwrap().guid()
            }));
        }
        let guids: Vec<_> = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect();
        assert!(guids.iter().all(|guid| *guid == guids[0]));
        assert_eq!(store.list().unwrap().len(), 1);
    }

    #[test]
    #[ignore = "subprocess helper invoked by the process-lock test"]
    fn process_create_worker() {
        let path = std::env::var_os("GW2_CHARACTER_STORE_TEST_DIR").expect("test directory");
        let store = CharacterStore::open(PathBuf::from(path)).unwrap();
        store.create(&test_request("Process retry")).unwrap();
    }

    #[test]
    fn separate_processes_wait_for_lock_and_share_one_record() {
        let temp = Temp::new();
        let store = temp.store();
        let lock = open_private(&temp.store_path().join(".lock"), true).unwrap();
        lock.lock().unwrap();
        let mut children = Vec::new();
        for _ in 0..4 {
            children.push(
                Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "character_store::tests::process_create_worker",
                        "--ignored",
                    ])
                    .env("GW2_CHARACTER_STORE_TEST_DIR", temp.store_path())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap(),
            );
        }
        std::thread::sleep(Duration::from_millis(100));
        assert!(
            children
                .iter_mut()
                .all(|child| child.try_wait().unwrap().is_none())
        );
        lock.unlock().unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        for child in &mut children {
            loop {
                if let Some(status) = child.try_wait().unwrap() {
                    assert!(status.success());
                    break;
                }
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("store subprocess timed out");
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        assert_eq!(store.list().unwrap().len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinks_and_public_permissions_without_repairing_them() {
        use std::os::unix::fs::symlink;
        let temp = Temp::new();
        let store = temp.store();
        let target = temp.0.join("outside");
        fs::write(&target, b"preserve").unwrap();
        let link = temp
            .store_path()
            .join(format!("{}.char", guid_hex(Guid([2; 16]))));
        symlink(&target, &link).unwrap();
        assert!(store.list().is_err());
        assert_eq!(fs::read(&target).unwrap(), b"preserve");
        fs::remove_file(link).unwrap();
        fs::set_permissions(temp.store_path(), fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(
            CharacterStore::open(temp.store_path())
                .err()
                .unwrap()
                .kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(
            fs::metadata(temp.store_path())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o755
        );
    }
}
