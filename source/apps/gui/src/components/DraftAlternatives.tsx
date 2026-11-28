import { gui } from '@/gen/catalog';
import { useSyncExternalStore } from 'react';
import { ChoiceField } from '@/components/ui/choice-field';
import { sessionDrafts } from '@/hooks/use-session-draft';

/** Concurrent offline edits remain selectable rather than silently choosing a
 * winner. Editing one alternative never erases another device's text. */
export function DraftAlternatives({ draftKey }: { draftKey: string }) {
  useSyncExternalStore(sessionDrafts.subscribe, sessionDrafts.version);
  const rows = sessionDrafts.alternatives(draftKey);
  if (rows.length < 2) return null;
  const selected = sessionDrafts.current(draftKey);
  return <div data-draft-alternatives className="mx-auto flex w-full max-w-measure items-center gap-in px-row py-in text-callout">
    <span className="flex-none text-muted-foreground">{gui.draft_alternatives(String(rows.length))}</span>
    <ChoiceField label={gui.choose_draft} value={selected?.text.replace(/\s+/g, ' ').slice(0, 80) ?? ''} selected={selected?.id}
      options={rows.map(row => ({ id: row.id, label: row.text.replace(/\s+/g, ' ').slice(0, 160) }))}
      onPick={id => sessionDrafts.choose(draftKey, id)} />
  </div>;
}
