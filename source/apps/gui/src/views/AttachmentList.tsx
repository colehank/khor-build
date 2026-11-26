import {useEffect,useState} from 'react';
import {fetchFileStat} from '@/api';
import type {Attachment} from '@/lib/term-attachments';
import {IconPlus} from '@/components/icons';
import {FileChoiceRow} from '@/views/FilesPane';

export type DraftAttachment={key:string;name:string;size:number|null;thumb?:string|null;dir?:boolean;detail?:string;source?:Attachment|null;error?:string|null;busy?:boolean};
/** A file or directory is one full-width row. Preview is optional;
 * selection and sending never require entering a preview. */
export function AttachmentList({items,onOpen,onRemove,onAdd,onRetry,busy=false}:{
 items:DraftAttachment[];onOpen:(key:string)=>void;onRemove:(key:string)=>void;
 onAdd?:()=>void;onRetry?:(key:string)=>void;busy?:boolean;
}){
 return <div data-attachment-list className="flex min-w-0 flex-col">
  {items.map(item=><AttachmentRow key={item.key} item={item} onOpen={()=>onOpen(item.key)} onRemove={()=>onRemove(item.key)} onRetry={onRetry?()=>onRetry(item.key):undefined} busy={busy}/>)}
  {onAdd&&<button type="button" data-attachment-add disabled={busy} onClick={onAdd} className="flex min-h-touch items-center gap-in text-left text-body text-muted-foreground"><IconPlus className="size-kind-mark"/>添加附件</button>}
 </div>;
}

/** A native drag carries a path, not bytes or a size. Fill that missing fact
 * from its source without staging a transfer or changing the selection. */
function AttachmentRow({item,onOpen,onRemove,onRetry,busy}:{item:DraftAttachment;onOpen:()=>void;onRemove:()=>void;onRetry?:()=>void;busy:boolean}){
 const source=item.source,dir=item.dir??source?.dir??false;
 const [fact,setFact]=useState<{key:string;size:number}|null>(null);
 useEffect(()=>{
  if(item.size!==null||!source||source.dir||source.device)return;
  let live=true;
  void fetchFileStat(source.from??'',source.path).then(stat=>{if(live&&Number.isFinite(stat.total)&&stat.total>=0)setFact({key:source.key,size:stat.total});},()=>{});
  return()=>{live=false;};
 },[item.size,source?.key,source?.path,source?.from,source?.dir,source?.device]);
 const size=item.size??(fact?.key===source?.key?fact?.size??null:null);
 return <div data-files-selected={item.name} data-chat-pending-file={item.name}>
  <FileChoiceRow name={item.name} dir={dir} size={size} detail={item.error?'发送失败':item.detail} selected disabled={busy||item.busy} onEnter={()=>dir?onOpen():onRemove()} onChoose={onRemove} onOpen={onOpen}/>
  {item.error&&onRetry&&<button type="button" disabled={busy} onClick={onRetry} className="min-h-touch px-in text-callout">重试</button>}
 </div>;
}
