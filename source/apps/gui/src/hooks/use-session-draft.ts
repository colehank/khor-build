import { useCallback, useSyncExternalStore, type Dispatch, type SetStateAction } from 'react';
import { fetchSessionDrafts, editSessionDraft } from '@/api';
import { SessionDraftStore, type DraftStorage } from '@/lib/session-drafts';
import { ReplicatedDraftStore } from '@/lib/replicated-drafts';
const storage = (): DraftStorage | null => { try { return window.localStorage; } catch { return null; } };
const disk = storage();
const legacy = new SessionDraftStore(disk);
export const sessionDrafts = new ReplicatedDraftStore(disk, { read: fetchSessionDrafts, write: editSessionDraft });
sessionDrafts.migrate(legacy.all());
let timer: ReturnType<typeof setTimeout> | null = null;
function schedule() {
  if (timer !== null) clearTimeout(timer);
  timer = setTimeout(() => { timer = null; void sessionDrafts.sync().catch(() => {}); }, 500);
}
export const checkpointSessionDrafts = (): boolean => sessionDrafts.durable();
export function syncSessionDrafts() { void sessionDrafts.sync().catch(() => {}); }
export function useSessionDraft(key: string): [string, Dispatch<SetStateAction<string>>] {
  useSyncExternalStore(sessionDrafts.subscribe, sessionDrafts.version);
  const update = useCallback<Dispatch<SetStateAction<string>>>(value => {
    sessionDrafts.set(key, typeof value === 'function' ? value(sessionDrafts.get(key)) : value);
    schedule();
  }, [key]);
  return [sessionDrafts.get(key), update];
}
