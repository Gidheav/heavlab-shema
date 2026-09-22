# Data Pack Format — SERMON (HV-Bible)

## Overview

Each Bible translation is distributed as a **data pack** consisting of two binary files. These are downloadable assets — never embedded in the app binary. Only the canonical structure index and alias tables are compiled into the app.

## File Types

### `*.bible.bin` — Verse Archive

Contains all verse text for a single translation, serialized with `rkyv`.

#### Layout

```
┌──────────────────────────────────────────────────────┐
│ Header (44 bytes)                                     │
│  ├─ Magic bytes: "HVBB" (4 bytes)                    │
│  ├─ Format version: u32 LE (4 bytes)                 │
│  ├─ Verse count: u32 LE (4 bytes)                    │
│  └─ BLAKE3 checksum of payload (32 bytes)            │
├──────────────────────────────────────────────────────┤
│ Payload (variable length)                             │
│  └─ rkyv-archived VerseRecord array                  │
│     Each record: { text: ArchivedString }            │
└──────────────────────────────────────────────────────┘
```

#### Validation Protocol

1. Read the 44-byte header
2. Verify magic bytes == `"HVBB"`
3. Verify format version == `PACK_FORMAT_VERSION` (currently 1)
4. Compute BLAKE3 over the payload bytes (everything after the header)
5. Compare to the stored checksum
6. If any check fails → `EngineError::CorruptDataPack`
7. Only after validation: access payload via `rkyv`'s unchecked accessor

### `*.offsets.bin` — Offset Table

Maps `GlobalVerseIndex` to byte offsets within the `.bible.bin` payload.

#### Layout

```
┌──────────────────────────────────────────────────────┐
│ Header (12 bytes)                                     │
│  ├─ Magic bytes: "HVBO" (4 bytes)                    │
│  ├─ Format version: u32 LE (4 bytes)                 │
│  └─ Entry count: u32 LE (4 bytes)                    │
├──────────────────────────────────────────────────────┤
│ Entries (entry_count × 8 bytes)                       │
│  Each entry:                                          │
│  ├─ Offset: u32 LE (byte position in payload)       │
│  └─ Length: u32 LE (byte length of archived record)  │
└──────────────────────────────────────────────────────┘
```

#### Access Pattern

```
GlobalVerseIndex → offsets[index] → (offset, length)
                                       │
                                       ▼
                              mmap[offset..offset+length]
                                       │
                                       ▼
                              rkyv::archived_root::<VerseRecord>(bytes)
                                       │
                                       ▼
                              &ArchivedString → &str (zero-copy)
```

## VerseRecord Schema

```rust
#[derive(rkyv::Archive, rkyv::Serialize)]
pub struct VerseRecord {
    /// The verse text content.
    pub text: String,
}
```

The `ArchivedVerseRecord` provides zero-copy access to the text via `rkyv`'s generated `ArchivedString` type.

## Versioning

- **Current format version:** 1
- Any change to the `VerseRecord` layout, header format, or serialization method requires incrementing the format version
- The app checks the version on load and returns `EngineError::UnsupportedPackVersion` on mismatch
- Old packs are never silently reinterpreted — they must be re-downloaded

## Integrity

- BLAKE3 chosen for speed (>1 GB/s on modern ARM) and cryptographic security
- Checksum is computed over the payload only (not the header) to allow header reads before full validation
- A failed checksum prevents any `rkyv` access — the pack is treated as corrupt

## Lifetime Safety

- The `.bible.bin` file is memory-mapped via `memmap2::Mmap`
- The `Mmap` is held inside `Arc<Mmap>` to allow shared ownership
- Borrowed data from the mmap uses `yoke::Yoke` to safely bundle the owner with borrowed slices
- No `'static` lifetime assertions on mmap-borrowed data
