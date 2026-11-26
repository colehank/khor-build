/** Local receipts cover the row version the person opened, never a future
 * completion. Keep them after persistence: an older list may still arrive. */
export type ReadReceipt = { id: string; at_ms: number };
type ReadRow = ReadReceipt & { unread: number };

export class SessionReads {
  private seen = new Map<string, number>();
  private queued = new Map<string, ReadReceipt>();
  private readable = true;
  private storage: Pick<Storage, "getItem" | "setItem"> | null;
  constructor(storage: Pick<Storage, "getItem" | "setItem"> | null = null) {
    this.storage = storage;
    try {
      const raw = storage?.getItem("khor.session-reads.v1");
      if (!raw) return;
      if (raw.length > 512 * 1024) throw Error("invalid read queue");
      const saved = JSON.parse(raw);
      const valid = (entries: unknown): entries is [string, number][] => Array.isArray(entries) && entries.length <= 8192 && entries.every(entry =>
        Array.isArray(entry) && entry.length === 2 && typeof entry[0] === "string" && Number.isSafeInteger(entry[1]) && entry[1] >= 0);
      if (!valid(saved.seen) || !valid(saved.queued)) throw Error("invalid read queue");
      this.seen = new Map(saved.seen);
      this.queued = new Map(saved.queued.map(([id, at_ms]: [string, number]) => [id, { id, at_ms }]));
    } catch { this.readable = false; }
  }
  private checkpoint() {
    if (!this.readable) return;
    try {
      this.storage?.setItem("khor.session-reads.v1", JSON.stringify({ seen: [...this.seen], queued: [...this.queued].map(([id, receipt]) => [id, receipt.at_ms]) }));
    } catch { /* Keep the live queue; a failed disk write is not a read receipt. */ }
  }

  read(row: ReadRow): ReadReceipt | null {
    // The shared conversation acknowledges individual rendered message versions,
    // not a session watermark (including messages that were never on screen).
    if (row.id.startsWith("wide/") || row.unread === 0 || !Number.isSafeInteger(row.at_ms) || row.at_ms < 0) return null;
    const at = this.seen.get(row.id);
    if (at !== undefined && at >= row.at_ms) return null;
    const receipt = { id: row.id, at_ms: row.at_ms };
    this.seen.set(row.id, row.at_ms);
    this.queued.set(row.id, receipt);
    this.checkpoint();
    return receipt;
  }

  project<T extends ReadRow>(rows: T[]): T[] {
    return rows.map((row) => {
      const at = this.seen.get(row.id);
      return at !== undefined && row.at_ms <= at && row.unread > 0 ? { ...row, unread: 0 } : row;
    });
  }

  pending(): ReadReceipt[] { return [...this.queued.values()]; }

  confirmed(receipt: ReadReceipt): void {
    // An earlier request completing cannot drop the newer queued receipt.
    if (this.queued.get(receipt.id)?.at_ms === receipt.at_ms) {
      this.queued.delete(receipt.id);
      this.checkpoint();
    }
  }
}
