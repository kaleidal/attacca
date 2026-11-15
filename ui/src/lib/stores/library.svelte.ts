import { commands, events, type PlaylistSummary, type Song } from "../bindings";
import { albumKey, artist, compare, title } from "../format";
import { forgetWaveform, setCoversFolder } from "../media";

export interface Album {
  key: string;
  title: string;
  artist: string;
  year: number | null;
  cover: string | null;
  songs: Song[];
  duration: number;
}

export interface Artist {
  name: string;
  songs: Song[];
  albums: Album[];
  cover: string | null;
}

const FULL_RELOAD_AFTER = 40;
const RELOAD_SETTLE_MS = 1200;

function groupAlbums(songs: Song[]): Album[] {
  const albums = new Map<string, Album>();
  for (const song of songs) {
    const key = albumKey(song);
    if (!key) continue;
    let album = albums.get(key);
    if (!album) {
      album = {
        key,
        title: song.album!,
        artist: song.albumArtist ?? song.artist ?? "",
        year: song.year,
        cover: song.cover,
        songs: [],
        duration: 0,
      };
      albums.set(key, album);
    }
    album.songs.push(song);
    album.duration += song.duration;
    album.cover ??= song.cover;
    album.year ??= song.year;
  }
  for (const album of albums.values()) {
    album.songs.sort((a, b) => (a.disc ?? 1) - (b.disc ?? 1) || (a.track ?? 0) - (b.track ?? 0) || compare(title(a), title(b)));
  }
  return [...albums.values()].sort((a, b) => compare(a.artist, b.artist) || compare(a.title, b.title));
}

function groupArtists(songs: Song[], albums: Album[]): Artist[] {
  const artists = new Map<string, Artist>();
  for (const song of songs) {
    const name = artist(song);
    let entry = artists.get(name.toLowerCase());
    if (!entry) {
      entry = { name, songs: [], albums: [], cover: null };
      artists.set(name.toLowerCase(), entry);
    }
    entry.songs.push(song);
    entry.cover ??= song.cover;
  }
  for (const album of albums) {
    artists.get(album.artist.toLowerCase())?.albums.push(album);
  }
  return [...artists.values()].sort((a, b) => compare(a.name, b.name));
}

class LibraryStore {
  songs = $state<Song[]>([]);
  playlists = $state<PlaylistSummary[]>([]);
  folders = $state<string[]>([]);
  scanning = $state(false);
  ready = $state(false);
  byId = $derived(new Map(this.songs.map((song) => [song.id, song])));
  albums = $derived(groupAlbums(this.songs));
  artists = $derived(groupArtists(this.songs, this.albums));
  albumsByKey = $derived(new Map(this.albums.map((album) => [album.key, album])));
  #reload: ReturnType<typeof setTimeout> | null = null;

  async start() {
    setCoversFolder(await commands.libraryCoversFolder());
    events.libraryChanged(({ changed, removed }) => this.#changed(changed, removed));
    events.libraryAnalysed((song) => this.#upsert([song]));
    events.libraryScanning(({ busy }) => (this.scanning = busy));
    events.playlistsChanged(() => this.loadPlaylists());
    await Promise.all([this.loadSongs(), this.loadPlaylists(), this.loadFolders()]);
    this.ready = true;
  }

  async loadSongs() {
    this.songs = await commands.librarySongs();
  }

  async loadPlaylists() {
    this.playlists = await commands.playlistsList();
  }

  async loadFolders() {
    this.folders = await commands.libraryFolders();
  }

  #changed(changed: string[], removed: string[]) {
    if (removed.length) {
      const gone = new Set(removed);
      this.songs = this.songs.filter((song) => !gone.has(song.id));
    }
    if (changed.length === 0 && removed.length === 0) {
      void this.loadSongs();
      void this.loadFolders();
      return;
    }
    changed.forEach(forgetWaveform);
    if (changed.length > FULL_RELOAD_AFTER) {
      if (this.#reload) clearTimeout(this.#reload);
      this.#reload = setTimeout(() => void this.loadSongs(), RELOAD_SETTLE_MS);
      return;
    }
    void this.#upsert(changed);
  }

  async #upsert(ids: string[]) {
    const fetched = await Promise.all(ids.map((id) => commands.librarySong({ id })));
    const next = [...this.songs];
    const index = new Map(next.map((song, position) => [song.id, position]));
    for (const song of fetched) {
      if (!song) continue;
      const position = index.get(song.id);
      if (position === undefined) next.unshift(song);
      else next[position] = song;
    }
    this.songs = next;
    void this.loadPlaylists();
  }

  async addFolder() {
    const folder = await commands.libraryAddFolder();
    if (folder) await this.loadFolders();
    return folder;
  }

  async removeFolder(path: string) {
    await commands.libraryRemoveFolder({ path });
    await this.loadFolders();
  }
}

export const library = new LibraryStore();
