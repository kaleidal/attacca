use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// The part of an MP4 audio track meant to be heard, from its edit list: encoders put priming
/// samples before the song and padding after it.
#[derive(Clone, Copy, Debug)]
pub struct Edit {
    pub lead: u64,
    pub frames: u64,
    /// Units per second of `lead` and `frames`.
    pub scale: u64,
}

struct Atom {
    kind: [u8; 4],
    body: u64,
    end: u64,
}

fn next_atom(file: &mut File, end: u64) -> Option<Atom> {
    let start = file.stream_position().ok()?;
    if start + 8 > end {
        return None;
    }
    let mut header = [0u8; 8];
    file.read_exact(&mut header).ok()?;
    let mut size = u64::from(u32::from_be_bytes(header[..4].try_into().ok()?));
    let kind = header[4..].try_into().ok()?;
    let mut body = start + 8;
    if size == 1 {
        let mut large = [0u8; 8];
        file.read_exact(&mut large).ok()?;
        size = u64::from_be_bytes(large);
        body += 8;
    } else if size == 0 {
        size = end - start;
    }
    (size >= body - start).then_some(Atom {
        kind,
        body,
        end: start + size,
    })
}

fn children(file: &mut File, parent: &Atom) -> Vec<Atom> {
    let mut atoms = Vec::new();
    if file.seek(SeekFrom::Start(parent.body)).is_err() {
        return atoms;
    }
    while let Some(atom) = next_atom(file, parent.end) {
        let end = atom.end;
        atoms.push(atom);
        if file.seek(SeekFrom::Start(end)).is_err() {
            break;
        }
    }
    atoms
}

fn read_at(file: &mut File, offset: u64, length: usize) -> Option<Vec<u8>> {
    file.seek(SeekFrom::Start(offset)).ok()?;
    let mut bytes = vec![0; length];
    file.read_exact(&mut bytes).ok()?;
    Some(bytes)
}

fn be(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .fold(0, |value, byte| value << 8 | u64::from(*byte))
}

/// Reads the first audio track's edit list, when the file has one.
pub fn edit(path: &Path) -> Option<Edit> {
    let mut file = File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    let root = Atom {
        kind: *b"root",
        body: 0,
        end: length,
    };
    let moov = children(&mut file, &root)
        .into_iter()
        .find(|atom| &atom.kind == b"moov")?;
    let atoms = children(&mut file, &moov);
    let mvhd = atoms.iter().find(|atom| &atom.kind == b"mvhd")?;
    let version = read_at(&mut file, mvhd.body, 1)?[0];
    let movie_scale = be(&read_at(
        &mut file,
        mvhd.body + if version == 1 { 20 } else { 12 },
        4,
    )?);
    for trak in atoms.iter().filter(|atom| &atom.kind == b"trak") {
        let parts = children(&mut file, trak);
        let Some(mdia) = parts.iter().find(|atom| &atom.kind == b"mdia") else {
            continue;
        };
        let media = children(&mut file, mdia);
        let is_audio = media
            .iter()
            .find(|atom| &atom.kind == b"hdlr")
            .and_then(|hdlr| read_at(&mut file, hdlr.body + 8, 4))
            .is_some_and(|handler| handler == b"soun");
        if !is_audio {
            continue;
        }
        let mdhd = media.iter().find(|atom| &atom.kind == b"mdhd")?;
        let version = read_at(&mut file, mdhd.body, 1)?[0];
        let media_scale = be(&read_at(
            &mut file,
            mdhd.body + if version == 1 { 20 } else { 12 },
            4,
        )?);
        let edts = parts.iter().find(|atom| &atom.kind == b"edts")?;
        let elst = children(&mut file, edts)
            .into_iter()
            .find(|atom| &atom.kind == b"elst")?;
        let header = read_at(&mut file, elst.body, 8)?;
        let (version, count) = (header[0], be(&header[4..8]));
        let entry = if version == 1 { 20 } else { 12 };
        let field = if version == 1 { 8 } else { 4 };
        for index in 0..count {
            let bytes = read_at(&mut file, elst.body + 8 + index * entry, entry as usize)?;
            let duration = be(&bytes[..field]);
            let media_time = be(&bytes[field..field * 2]);
            let empty = if version == 1 {
                media_time == u64::MAX
            } else {
                media_time == u64::from(u32::MAX)
            };
            if empty || movie_scale == 0 {
                continue;
            }
            let frames = duration * media_scale / movie_scale;
            return (media_time > 0 || frames > 0).then_some(Edit {
                lead: media_time,
                frames,
                scale: media_scale,
            });
        }
        return None;
    }
    None
}
