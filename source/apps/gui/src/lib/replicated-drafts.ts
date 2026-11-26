import type { DraftStorage } from './session-drafts';
import type { DraftRevision } from '@/api';

type SubmissionPhase = 'pending' | 'failed' | 'cancelled' | 'unconfirmed';
type Edit = { revision: DraftRevision; parents: string[] };
type Transport = { read: () => Promise<DraftRevision[]>; write: (revision: DraftRevision, parents: string[]) => Promise<DraftRevision[]> };
const KEY = 'khor.replicated-drafts.v1';
const LIMIT = 2 * 1024 * 1024;

function isDraftRevision(row: unknown): row is DraftRevision {
  if (!row || typeof row !== 'object') return false;
  const candidate = row as Partial<DraftRevision>;
  return typeof candidate.id === 'string' && typeof candidate.key === 'string' &&
    /^(gui|khor|term):.+/.test(candidate.key) && typeof candidate.text === 'string' &&
    Number.isSafeInteger(candidate.at_ms);
}

function checkedRows(rows: unknown): DraftRevision[] {
  if (!Array.isArray(rows) || !rows.every(isDraftRevision)) throw Error('Invalid draft sync response');
  return rows;
}

/** The editor and its delivery queue have independent lifetimes. Only observed
 * revisions are replaced; concurrently edited alternatives stay available. */
export class ReplicatedDraftStore {
  private remote: DraftRevision[] = [];
  private pending: Edit[] = [];
  private selected = new Map<string, string>();
  private listeners = new Set<() => void>();
  private serial = 0;
  private migrated = false;
  private submissions = new Map<string, { key: string; id: string; phase: SubmissionPhase; held?: boolean }>();
  private sending: Edit | null = null;
  private syncing = false;
  private readable = true;
  private writable = true;

  private storage: DraftStorage | null;
  private transport: Transport;
  private mint: () => string;
  private now: () => number;
  constructor(storage: DraftStorage | null, transport: Transport,
    mint: () => string = () => crypto.randomUUID(), now: () => number = Date.now) {
    this.storage = storage; this.transport = transport; this.mint = mint; this.now = now;
    try {
      const raw = storage?.getItem(KEY);
      if (!raw) return;
      if (raw.length > LIMIT) throw Error('draft storage too large');
      const parsed = JSON.parse(raw);
      const remote = checkedRows(parsed.remote);
      if (!Array.isArray(parsed.pending) ||
        !parsed.pending.every((edit: Edit) => isDraftRevision(edit.revision) && Array.isArray(edit.parents) && edit.parents.every(id => typeof id === 'string')) ||
        !Array.isArray(parsed.selected) || !parsed.selected.every((pair: unknown) => Array.isArray(pair) && pair.length === 2 && pair.every(value => typeof value === 'string'))) throw Error('invalid draft storage');
      if (parsed.submissions !== undefined && (!Array.isArray(parsed.submissions) || !parsed.submissions.every((entry: unknown) =>
        Array.isArray(entry) && entry.length === 2 && typeof entry[0] === 'string' && entry[1] && typeof entry[1].id === 'string' && typeof entry[1].key === 'string'))) throw Error('invalid submission receipts');
      this.submissions = new Map((parsed.submissions ?? []).map(([token, value]: [string, {key:string;id:string}]) => [token, {...value, phase:'unconfirmed' as const}]));
      this.migrated = parsed.migrated === true;
      this.remote = remote;
      this.pending = parsed.pending;
      this.selected = new Map(parsed.selected);
    } catch { this.readable = false; }
  }

  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; };
  version = () => this.serial;
  durable = () => this.pending.length === 0 || this.readable && this.writable;
  private changed() {
    this.serial++;
    const raw = JSON.stringify({ submissions: [...this.submissions], migrated: this.migrated, remote: this.remote, pending: this.pending, selected: [...this.selected] });
    this.writable = false;
    if (this.readable && raw.length <= LIMIT && this.storage) {
      try { this.storage.setItem(KEY, raw); this.writable = true; } catch { /* Retain in memory and report unsaved state. */ }
    }
    for (const listener of this.listeners) listener();
  }

  all(): DraftRevision[] {
    const rows = new Map(this.remote.map(row => [row.id, row]));
    for (const edit of this.pending) {
      for (const parent of edit.parents) rows.delete(parent);
      if (edit.revision.text) rows.set(edit.revision.id, edit.revision);
    }
    const hidden = new Set([...this.submissions.values()].filter(entry => entry.phase === 'pending' || entry.held).map(entry => entry.id));
    return [...rows.values()].filter(row => !hidden.has(row.id));
  }
  alternatives(key: string): DraftRevision[] {
    return this.all().filter(row => row.key === key).sort((a, b) => b.at_ms - a.at_ms || a.id.localeCompare(b.id));
  }
  current(key: string): DraftRevision | null {
    const rows = this.alternatives(key);
    return rows.find(row => row.id === this.selected.get(key)) ?? rows[0] ?? null;
  }
  get(key: string): string { return this.current(key)?.text ?? ''; }
  choose(key: string, id: string) {
    if (!this.alternatives(key).some(row => row.id === id)) return;
    this.selected.set(key, id);
    this.changed();
  }
  set(key: string, text: string, force = false): void {
    if (!force && text === this.get(key)) return;
    const current = this.current(key);
    let parents = current ? [current.id] : [];
    // Coalesce edits not yet sent, but never mutate an in-flight snapshot.
    const previous = this.pending.find(edit => edit.revision.id === current?.id && edit !== this.sending);
    if (previous) {
      parents = previous.parents;
      this.pending = this.pending.filter(edit => edit !== previous);
    }
    // Creation raises attention once; subsequent keystrokes update content,
    // not the draft's position among other drafts.
    const revision = { id: this.mint(), key, text, at_ms: current?.at_ms ?? this.now() };
    this.pending.push({ revision, parents });
    this.selected.set(key, revision.id);
    this.changed();
  }
  /** Exact revision receipt: a newer edit or a concurrent device is untouched. */
  clear(key: string, id: string) {
    const revision = { id: this.mint(), key, text: '', at_ms: this.now() };
    this.pending.push({ revision, parents: [id] });
    this.changed();
  }
  begin(key: string, token: string, revision?: string | null) {
    const id = revision === undefined ? this.current(key)?.id : revision;
    if (id) this.submissions.set(token, { key, id, phase: 'pending' });
    this.changed();
  }
  ownsEditor(token: string) {
    const submitted = this.submissions.get(token);
    return !!submitted && this.current(submitted.key)?.id === submitted.id;
  }
  retry(previous: string, token: string) {
    const submitted = this.submissions.get(previous);
    if (submitted) this.submissions.set(token, { ...submitted, phase: 'pending', held: false });
    this.changed();
  }
  committed(token: string) {
    const submitted = this.submissions.get(token);
    if (!submitted) return;
    for (const [held, snapshot] of this.submissions) if (snapshot.key === submitted.key && snapshot.id === submitted.id) this.submissions.delete(held);
    this.clear(submitted.key, submitted.id);
  }
  failed(token: string, phase: 'failed' | 'cancelled' = 'failed', held = false) {
    const submitted = this.submissions.get(token);
    if (!submitted) return;
    // ChatView retains these sends with their attachments in its recovery UI.
    // Keep the same revision available to restore without replacing an edit.
    submitted.phase = phase;
    submitted.held = held;
    this.changed();
  }
  migrate(entries: readonly (readonly [string, string])[]) {
    if (this.migrated) return;
    for (const [key, text] of entries) if (text && !this.alternatives(key).some(row => row.text === text)) this.set(key, text);
    this.migrated = true;
    this.changed();
  }
  async sync(): Promise<void> {
    if (this.syncing) return;
    this.syncing = true;
    try {
      while (this.pending.length) {
        const edit = this.pending[0];
        this.sending = edit;
        const rows = checkedRows(await this.transport.write(edit.revision, edit.parents));
        this.remote = rows;
        this.pending = this.pending.filter(entry => entry !== edit);
        this.sending = null;
        this.changed();
      }
      const rows = checkedRows(await this.transport.read());
      // Revisions are immutable. An unchanged poll is neither an editor event
      // nor a reason to rewrite storage and repaint every retained chat.
      const identity = (values: DraftRevision[]) => values.map(row => row.id).sort().join('\n');
      if (identity(rows) !== identity(this.remote)) { this.remote = rows; this.changed(); }
    } finally { this.sending = null; this.syncing = false; }
  }
}
