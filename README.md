# Lyricshop

A lyrics and songwriting-structure tool, written in Rust with
[egui](https://github.com/emilk/egui). It is built for writing lyrics and
shaping a song's structure, with chord playback as a listening aid and a
rhyme assistant built in.

## Build and run

```sh
cargo run
```

Or build first, then run the binary:

```sh
cargo build
./target/debug/lyricshop
```

Needs a display (X11/Wayland on Linux, or a Mac/Windows desktop) and an
audio output device for chord playback. The app runs on any desktop;
chord playback plays through the system audio output.

Requires Rust 1.85+. The first build downloads the dependencies from
crates.io (see Dependencies below).

## Dependencies

| Project | Used for |
|---|---|
| [eframe](https://github.com/emilk/egui) 0.32 | Application framework: window, event loop, rendering |
| [egui](https://github.com/emilk/egui) 0.32 | Immediate-mode GUI: menus, dialogs, text fields, panels |
| [serde](https://serde.rs) 1 + [serde_json](https://github.com/serde-rs/json) 1 | JSON save/load of songs |
| [rodio](https://github.com/RustAudio/rodio) 0.21 | Audio output for chord playback |
| [rfd](https://github.com/PolyMeilex/rfd) 0.15 | Native file dialogs (Open, Save As) |
| [image](https://github.com/image-rs/image) 0.25 (PNG) | Decodes `assets/icon.png` for the window icon |
| [winres](https://github.com/mcgoo/winres) 0.1 (Windows build) | Embeds `assets/icon.ico` into the `.exe` |
| [CMU Pronouncing Dictionary](http://www.speech.cs.cmu.edu/cgi-bin/cmudict) | Rhyme and syllable data (`assets/cmudict.dict`, ~135k words), embedded in the binary |

## Features

- **Song sections** — Intro, Verse, Pre-Chorus, Chorus, Post-Chorus, Bridge,
  Solo, Outro, Custom. Move up/down, lock, delete; add any type from
  the ＋ Add Section dialog.
- **Duplicate section** — copy a section (label, type, time signature,
  chords, lyrics) right after the original; the copy starts unlocked.
- **Genre templates** — 8 core genres plus 100+ advanced styles; a new song
  generates the matching section list (Blues and Jazz get their own shapes,
  Custom/Minimal is a single Verse).
- **JSON save/load** — Save (Ctrl/Cmd+S), Save As, Open (Ctrl/Cmd+O),
  New (Ctrl/Cmd+N).
- **Dirty tracking + 2s autosave** — edits mark the song dirty; 2 seconds
  after the last edit it autosaves to the already-known file. Autosave
  begins once the song has been saved to a file. Closing with unsaved
  changes asks first.
- **Undo/redo** — the lyrics fields use egui's built-in undo (Ctrl+Z,
  Ctrl+Shift+Z / Ctrl+Y).
- **Find & replace** — Edit → Find & Replace… (Ctrl+H); case-insensitive
  replace-all across every section's lyrics, with a live match count.
- **Word counts** — per-section counts under each lyrics box; the status bar
  shows the whole-song total and section count.
- **BPM validation** — accepts 20–400. While you type, the last valid BPM
  stays until a value in range commits.
- **Chord playback** — synthesizes each chord as mixed sine waves through
  rodio. Loops until stopped. Transpose −6…+6 semitones, playback-only
  (−2 is labelled "D std."). The waveform selector lists the three
  waveforms the synth produces (Sine Wave, Triangle Wave, Sawtooth Wave).
  **Power chords ('5') play root–fifth–octave** (B5 = B–F#–B,
  MIDI 71, 78, 83).
- **Chord suggestions** — diatonic chord chips for the song key (tap to
  append), and a Fill button cycling classic progressions (I–V–vi–IV first
  for major, i–VI–III–VII first for minor).
- **CMU rhyme assistant** — the CMU Pronouncing Dictionary
  (`assets/cmudict.dict`, ~135k words) is **embedded in the binary**, so
  rhymes work out of the box. Perfect / Slant / Assonance
  modes, ranked by syllable-count match, then common lyric words, then
  length, then alphabetical; top 30. The search box is
  seeded with the last word you typed; clicking a rhyme inserts it into
  the last-edited section. The dictionary holds ordinary words for
  rhyming.
- **Rhyme syllable filter** — filter the rhyme results by syllable count
  (Any / 1 / 2 / 3 / 4 / 5+).
- **Syllable / meter** — per-line syllable counts from the dictionary
  (vowel-group estimate for unknown words), with the usual line length and
  outlier lines called out.
- **Rhyme scheme** — A B A B… from the last word of each line, grouped by
  perfect rhyme.
- **Snapshots** — timestamped JSON copies in a `snapshots` folder next to the
  song file; restore or delete from File → Restore Snapshot. Snapshots
  work once the song has been saved to a file.
- **Song library** — File → Song Library lists every `.json` song in the
  same folder as the current song, newest first, searchable by title or
  genre; click to open.
- **PDF export** — hand-rolled: Letter page, Helvetica,
  title + metadata + one block per section, with pagination and ASCII
  flattening.
- **TXT export** — plain-text export of the song (title, metadata, one
  block per section).

## Theme

Dark is the default (the look is built with egui's own `Visuals`):

| | Dark (default) | Light |
|---|---|---|
| Editor / background | `#111113` | `#ffffff` |
| Chrome / panels | `#232328` | `#e9e9ec` |
| Card | `#1b1b1f` | `#f4f4f6` |
| Text | `#e8e8ea` | `#1a1a1a` |
| Accent | `#1a73e8` | `#1a73e8` |

Switch from the **View → Theme** menu. The choice is remembered in a
preferences file (`~/.config/lyricshop/preferences.json` on Linux/macOS,
`%APPDATA%\Lyricshop\preferences.json` on Windows) and survives restarts.

## Layout of the code

One concern per file:

| File | What it is |
|---|---|
| `src/main.rs` | eframe entry point |
| `src/app.rs` | App state, menu bar, shortcuts, autosave, close confirm |
| `src/ui.rs` | Setup dialog, header, section cards, rhyme panel, dialogs |
| `src/song.rs` | Data model + genre templates |
| `src/serialization.rs` | JSON save/load |
| `src/chords.rs` | Chord parsing (incl. power chords), bars, transpose |
| `src/chord_suggestions.rs` | Key parsing, diatonic chords, progressions |
| `src/player.rs` | rodio sine synthesis, looping playback |
| `src/rhyme.rs` | CMU rhyme engine (embedded dictionary) |
| `src/syllables.rs` | Syllable counts + meter |
| `src/rhyme_scheme.rs` | Rhyme-scheme detection |
| `src/pdf.rs` | Hand-rolled PDF exporter |
| `src/snapshots.rs` | Snapshot save/list/restore/delete |
| `src/library.rs` | Song library listing + search |
| `src/storage.rs` | File save/load/open, TXT export |
| `src/wordcount.rs` | Word counts |
| `src/theme.rs` | Dark + light palettes → egui Visuals |
| `src/prefs.rs` | Preferences file (theme choice) |

## Tests

```sh
cargo test
```

89 unit tests cover the pure logic: chord parsing (including power chords),
transpose, time signatures, syllable counting, rhyme lookup
(perfect/slant/assonance), rhyme scheme, JSON round-trip, PDF building,
snapshots, library, theme palettes, preferences, duplicate section,
find & replace, the rhyme syllable filter, icon geometry, viewport sizes,
and top-bar alignment.

The tests cover the pure logic. The GUI runs on a machine with a display.

## License

Lyricshop is licensed under the GNU General Public License v3.0 (or later).
See `LICENSE` for the full text.
