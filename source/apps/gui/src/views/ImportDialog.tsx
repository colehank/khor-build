/** All ACP providers share the source-owned catalog and action contract. */
import {useEffect,useMemo,useRef,useState} from 'react';
import {fetchDevices,sessionImport,type DeviceRow,type ImportCatalog,type ImportRow,type SessionImportReply} from '@/api';
import {ImportCalm,ImportMachinePicker,ImportResult,ImportSearch,importMachineAway,importMachineInScope,toggleImportMachine} from '@/components/ImportPicker';
import {sessionMark} from '@/components/BrandIcons';
import {Button} from '@/components/ui/button';
import {Dialog,TaskOverlay,TaskActions,DialogBody,DialogHeader,DialogTitle} from '@/components/ui/dialog';
import {agentPresentation} from '@/lib/acp-session';
import {gui} from '@/gen/catalog';
import {imeKey} from '@/lib/ime';
import {ago,basename} from '@/words';
import {claimImportIntent,settleImportIntent} from '@/lib/session-import-intent';

const rowKey=(home:string,row:ImportRow)=>JSON.stringify([home,row.agent,row.native_id]);
export function importCatalogFrom(home:string,reply:SessionImportReply):ImportCatalog{
  if(reply.kind!=='catalog'||reply.value.home!==home)throw new Error(gui.import_source_changed);
  return reply.value;
}
function unavailable(reason:string|null):string{
  switch(reason){
    case 'stale':return gui.import_refresh_required;
    case 'archive_unknown':return gui.import_archive_unknown;
    case 'directory_unknown':return gui.import_directory_unknown;
    case 'running_elsewhere':return gui.import_running_elsewhere;
    case 'restore_unsupported':return gui.import_restore_unsupported;
    default:return gui.import_owner_unknown;
  }
}

export function ImportDialog({open,onOpenChange,onImported}:{
  open:boolean;onOpenChange:(open:boolean)=>void;
  onImported:(ids:string[],open?:boolean)=>void;
}){
  const [devices,setDevices]=useState<DeviceRow[]>([]);
  const [catalogs,setCatalogs]=useState<Record<string,ImportCatalog>>({});
  const [failures,setFailures]=useState<Record<string,string>>({});
  const [reading,setReading]=useState(true);
  const [query,setQuery]=useState('');
  const [picked,setPicked]=useState<Set<string>>(new Set());
  const [active,setActive]=useState(0);
  const [busy,setBusy]=useState<string|null>(null);
  const [error,setError]=useState<string|null>(null);
  const generation=useRef(0),field=useRef<HTMLInputElement>(null);
  const deliveries=useRef(new Map<string,string>());
  const pending=useRef(false);
  const why=(error:unknown)=>String(error instanceof Error?error.message:error);

  useEffect(()=>{
    if(!open)return;
    const epoch=++generation.current;
    setQuery('');setPicked(new Set());setActive(0);setError(null);setCatalogs({});setFailures({});setReading(true);setBusy(null);pending.current=false;
    void fetchDevices().then(async machines=>{
      if(epoch!==generation.current)return;
      machines=machines.filter(machine=>machine.hosts&&!machine.sponsor);
      setDevices(machines);
      await Promise.allSettled(machines.filter(machine=>machine.me||importMachineAway(machine)===null).map(async machine=>{
        try{
          const reply=await sessionImport(machine.id,{kind:'read'});
          const value=importCatalogFrom(machine.id,reply);
          if(epoch===generation.current)setCatalogs(was=>({...was,[machine.id]:value}));
        }catch(error){if(epoch===generation.current)setFailures(was=>({...was,[machine.id]:why(error)}));}
      }));
    }).catch(error=>{if(epoch===generation.current)setError(why(error));})
      .finally(()=>{if(epoch===generation.current)setReading(false);});
    return()=>{generation.current++;};
  },[open]);
  useEffect(()=>setActive(0),[query,picked]);

  const scoped=devices.filter(machine=>importMachineInScope(machine.id,picked,machine));
  const words=query.toLocaleLowerCase().trim().split(/\s+/).filter(Boolean);
  const rows=useMemo(()=>scoped.flatMap(machine=>(catalogs[machine.id]?.rows??[]).map(row=>({machine,row})))
    .filter(({machine,row})=>words.every(word=>[machine.shown,row.title,row.cwd,agentPresentation(row.agent).name].some(value=>value.toLocaleLowerCase().includes(word)))),[catalogs,devices,picked,query]);
  const refresh=async(home:string,agent:string)=>{
    if(pending.current)return;
    pending.current=true;const epoch=generation.current;setBusy(JSON.stringify([home,agent]));setError(null);
    try{
      const reply=await sessionImport(home,{kind:'refresh',agent});
      const value=importCatalogFrom(home,reply);
      if(epoch===generation.current)setCatalogs(was=>({...was,[home]:value}));
    }catch(error){if(epoch===generation.current)setError(why(error));}
    finally{if(epoch===generation.current){pending.current=false;setBusy(null);}}
  };
  const choose=async(home:string,row:ImportRow)=>{
    if(pending.current||!row.intent)return;
    pending.current=true;const epoch=generation.current,key=rowKey(home,row),intent=row.intent;
    const deliveryKey=JSON.stringify([key,intent]);
    setBusy(key);setError(null);
    try{
      // Keep this window's exact delivery even if another window has already
      // received and settled the shared durable token.
      const token=deliveries.current.get(deliveryKey)??await claimImportIntent(home,row.agent,row.native_id,intent);
      deliveries.current.set(deliveryKey,token);
      if(epoch!==generation.current)return;
      const reply=await sessionImport(home,{kind:'apply',agent:row.agent,native_id:row.native_id,intent,token});
      if(reply.kind!=='imported'||reply.home!==home||reply.intent!==intent)throw new Error(gui.import_source_changed);
      await settleImportIntent(home,row.agent,row.native_id,intent,token);
      deliveries.current.delete(deliveryKey);
      if(epoch===generation.current){onOpenChange(false);onImported([reply.id],intent==='fork');}
    }catch(error){if(epoch===generation.current)setError(why(error));}
    finally{if(epoch===generation.current){pending.current=false;setBusy(null);}}
  };

  return <Dialog open={open} onOpenChange={value=>{if(!pending.current)onOpenChange(value);}}>
    <TaskOverlay purpose="selection" scrollable data-import-dialog>
      <DialogHeader><DialogTitle>{gui.import_title}</DialogTitle></DialogHeader>
      <DialogBody data-import-body>
        <ImportSearch inputRef={field} label={gui.import_search} value={query} onChange={setQuery} onKeyDown={event=>{
          if(imeKey(event))return;
          if(event.key==='ArrowDown'){event.preventDefault();setActive(index=>Math.min(rows.length-1,index+1));}
          else if(event.key==='ArrowUp'){event.preventDefault();setActive(index=>Math.max(0,index-1));}
          else if(event.key==='Enter'&&rows[active]){event.preventDefault();void choose(rows[active].machine.id,rows[active].row);}
        }}/>
        {devices.length>0&&<div data-import-machines><ImportMachinePicker machines={devices.map(device=>({id:device.id,name:device.shown,device}))} picked={picked} onPick={id=>setPicked(was=>toggleImportMachine(was,id))} disabled={busy!==null}/></div>}
        <div data-import-rows className="flex min-w-0 flex-col">
          {reading&&rows.length===0?<ImportCalm sentence={gui.import_reading} mark={{'data-import-reading':''}}/>:
            rows.length===0?<ImportCalm sentence={query?gui.no_matches:gui.import_empty} mark={{'data-import-empty':''}}/>:
            rows.map(({machine,row},index)=>{
              const Glyph=sessionMark('observe',row.agent);
              return <ImportResult key={rowKey(machine.id,row)} data-import-row={rowKey(machine.id,row)}
                data-import-agent={row.agent} data-import-intent={row.intent??'unavailable'}
                glyph={<Glyph className="size-5"/>} title={row.title||basename(row.cwd)||agentPresentation(row.agent).name}
                command={row.intent==='fork'?gui.import_fork:row.intent==='import'?gui.import_sessions:undefined}
                detailTitle={row.cwd} detail={<>{machine.shown} · {agentPresentation(row.agent).name} · {row.cwd}</>} note={row.intent?undefined:unavailable(row.reason)}
                active={query!==''&&index===active} busy={busy===rowKey(machine.id,row)} disabled={!row.intent||busy!==null}
                onClick={()=>void choose(machine.id,row)}/>;
            })}
        </div>
        {scoped.map(machine=><section key={machine.id} data-import-source={machine.id} className="flex min-w-0 flex-col gap-xxs border-t pt-row">
          <h3 className="text-body font-title">{machine.shown}</h3>
          {failures[machine.id]&&<span className="text-callout text-destructive">{failures[machine.id]}</span>}
          {(catalogs[machine.id]?.agents??[]).map(agent=><div key={agent.agent} data-agent-seen={agent.agent} data-said={agent.status} className="flex min-w-0 items-center gap-row">
            <span className="min-w-0 flex-1 truncate text-body">{agentPresentation(agent.agent).name}
              <span className="ml-row text-caption text-muted-foreground">{agent.status==='unasked'?gui.new_session_seen_unasked:agent.status==='cannot'?gui.new_session_seen_cannot:agent.at_ms?ago(agent.at_ms):''}</span>
            </span>
            <Button size="sm" variant="ghost" data-agent-ask={agent.agent} disabled={busy!==null} aria-busy={busy===JSON.stringify([machine.id,agent.agent])||undefined} onClick={()=>void refresh(machine.id,agent.agent)}>{busy===JSON.stringify([machine.id,agent.agent])?gui.new_session_seen_asking:gui.new_session_seen_ask}</Button>
          </div>)}
        </section>)}
        {error&&<span data-import-error role="alert" className="text-callout text-destructive">{error}</span>}
      </DialogBody>
      <TaskActions busy={busy!==null} close={<Button data-dialog-cancel className="min-h-touch" onClick={()=>onOpenChange(false)}>{gui.close}</Button>}/>
    </TaskOverlay>
  </Dialog>;
}
