import assert from 'node:assert/strict';
import { ReplicatedDraftStore } from '../src/lib/replicated-drafts.ts';
const disk = () => { const data = new Map(); return { getItem: key => data.get(key) ?? null, setItem: (key, value) => data.set(key, value) }; };
const server = new Map(), receipts = new Set();
let sequence = 0, clock = 0, offline = false;
const transport = {
  read: async () => [...server.values()],
  write: async (revision, parents) => {
    if (offline) throw Error('offline');
    if (!receipts.has(revision.id)) {
      for (const parent of parents) server.delete(parent);
      if (revision.text) server.set(revision.id, revision);
      receipts.add(revision.id);
    }
    return [...server.values()];
  },
};
const store = (storage = disk(), wire = transport) => new ReplicatedDraftStore(storage, wire, () => `revision-${++sequence}`, () => ++clock);
const a = store(), b = store();
a.set('gui:a', 'base'); await a.sync(); await b.sync();
a.set('gui:a', 'phone offline edit'); b.set('gui:a', 'desktop offline edit');
await a.sync(); await b.sync(); await a.sync();
assert.deepEqual(a.alternatives('gui:a').map(row => row.text).sort(), ['desktop offline edit', 'phone offline edit']);
a.choose('gui:a', a.alternatives('gui:a').find(row => row.text.startsWith('phone')).id);
a.set('gui:a', 'revised phone'); await a.sync(); await b.sync();
assert.deepEqual(b.alternatives('gui:a').map(row => row.text).sort(), ['desktop offline edit', 'revised phone']);

const durable = disk(), c = store(durable);
offline = true; c.set('gui:offline', 'survives closing the app');
await assert.rejects(c.sync());
const reopened = store(durable); assert.equal(reopened.get('gui:offline'), 'survives closing the app');
offline = false; await reopened.sync();
assert.equal(server.size, 3);

// Submission visibility is separate from immutable delivery. In particular an
// ABA edit (same text again) still has a new identity and must survive old ack.
const d = store(); d.set('gui:send', 'same text'); await d.sync();
d.begin('gui:send', 'send-token'); assert.equal(d.get('gui:send'), '');
d.set('gui:send', 'other text'); d.set('gui:send', 'same text');
d.committed('send-token'); await d.sync();
assert.equal(d.get('gui:send'), 'same text');
assert.equal([...server.values()].filter(row => row.key === 'gui:send').length, 1);
d.begin('gui:send', 'failure'); d.failed('failure'); assert.equal(d.get('gui:send'), 'same text');

let release;
const lagging = store(disk(), { ...transport, write: (revision, parents) => new Promise(resolve => { release = async () => resolve(await transport.write(revision, parents)); }) });
lagging.set('gui:late', 'first'); const inFlight = lagging.sync();
lagging.set('gui:late', 'second');
await release(); await new Promise(resolve => setImmediate(resolve));
assert.equal(lagging.get('gui:late'), 'second', 'a write reply is not an assignment to the current editor');
await release(); await inFlight;
assert.equal(lagging.get('gui:late'), 'second');

const migratedDisk = disk(), old = store(migratedDisk);
old.migrate([['gui:legacy', 'old draft']]); await old.sync();
const legacyId = old.current('gui:legacy').id; old.clear('gui:legacy', legacyId); await old.sync();
const migratedAgain = store(migratedDisk); migratedAgain.migrate([['gui:legacy', 'old draft']]);
assert.equal(migratedAgain.get('gui:legacy'), '', 'one-time migration cannot resurrect sent text');
console.log('PASS: replicated editor alternatives, offline restart, immutable submissions, late replies and migration');

const submissionDisk=disk(), submitted=store(submissionDisk);
submitted.set('gui:restart-send','sent but unconfirmed');await submitted.sync();
submitted.begin('gui:restart-send','restart-token');
const afterRestart=store(submissionDisk);
assert.equal(afterRestart.get('gui:restart-send'),'sent but unconfirmed','unknown outcome remains recoverable after restart');
afterRestart.set('gui:restart-send','newer input');
afterRestart.committed('restart-token');await afterRestart.sync();
assert.equal(afterRestart.get('gui:restart-send'),'newer input','a replayed receipt after restart clears only its old snapshot');

const stable=store();stable.set('gui:position','start');
const created=stable.current('gui:position').at_ms;
stable.set('gui:position','continued typing');
assert.equal(stable.current('gui:position').at_ms,created,'typing does not continuously move a drafted row');

const preparing=store();preparing.set('gui:prepare','snapshot before upload');
const preparingId=preparing.current('gui:prepare').id;
preparing.begin('gui:prepare','upload-token',preparingId);
preparing.set('gui:prepare','typed during upload');
preparing.committed('upload-token');await preparing.sync();
assert.equal(preparing.get('gui:prepare'),'typed during upload');
const staleId=preparing.current('gui:prepare').id;
preparing.set('gui:prepare','another current revision');
preparing.begin('gui:prepare','stale-render',staleId);
assert.equal(preparing.get('gui:prepare'),'another current revision','a stale render cannot hide current input');
preparing.committed('stale-render');await preparing.sync();
assert.equal(preparing.get('gui:prepare'),'another current revision');
