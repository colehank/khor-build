import type {ImportCatalog} from '../../src/gen/bindings/ImportCatalog';
import type {ImportIntent} from '../../src/gen/bindings/ImportIntent';
import type {SessionImportAction} from '../../src/gen/bindings/SessionImportAction';
import type {SessionImportReply} from '../../src/gen/bindings/SessionImportReply';

/** Closed protocol data only. DSH does not claim archive/fork capabilities. */
export function sessionImportFixture(homes:readonly string[]){
  const managed=new Map<string,{home:string;id:string;agent:string;title:string;intent:ImportIntent}>();
  const calls:{home:string;action:SessionImportAction}[]=[];
  const receipts=new Map<string,{fingerprint:string;value:SessionImportReply}>();
  const catalogs=new Map<string,ImportCatalog>(homes.map(home=>[home,{home,at_ms:Date.now(),
    agents:['claude-acp','codex-acp','dsh','generic-acp'].map(agent=>({agent,at_ms:Date.now(),status:'sessions',reason:null,load:agent!=='dsh',resume:true,fork:agent==='codex-acp',archive:agent!=='dsh'})),
    rows:[
      {agent:'claude-acp',native_id:'original-a',title:'项目讨论',cwd:'/preview/project',updated_at:'2026-11-17T08:00:00Z',intent:'import',reason:null},
      {agent:'claude-acp',native_id:'original-b',title:'项目讨论',cwd:'/preview/other',updated_at:'2026-11-17T07:00:00Z',intent:'import',reason:null},
      {agent:'codex-acp',native_id:'external-running',title:'其他宿主运行中的任务',cwd:'/preview/project',updated_at:null,intent:'fork',reason:null},
      {agent:'dsh',native_id:'dsh-history',title:'DeepSeek Harness 历史',cwd:'/preview/project',updated_at:null,intent:null,reason:'archive_unknown'},
      {agent:'generic-acp',native_id:'generic-history',title:'自定义智能体历史',cwd:'/preview/project',updated_at:null,intent:'import',reason:null},
      {agent:'generic-acp',native_id:'unknown-owner',title:'占用状态未知',cwd:'/preview/project',updated_at:null,intent:null,reason:'owner_unknown'},
    ]
  } satisfies ImportCatalog]));
  return {
    rows:()=>[...managed.values()],
    calls,
    request(home:string,action:SessionImportAction):SessionImportReply{
      calls.push(structuredClone({home,action}));
      const value=catalogs.get(home);if(!value)throw new Error('Unknown import source');
      if(action.kind==='read')return {kind:'catalog',value:structuredClone(value)};
      if(action.kind==='refresh'){
        if(!value.agents.some(agent=>agent.agent===action.agent))throw new Error('Unknown import agent');
        value.at_ms=Date.now();return {kind:'catalog',value:structuredClone(value)};
      }
      const fingerprint=JSON.stringify([home,action.agent,action.native_id,action.intent]);
      const previous=receipts.get(action.token);
      if(previous){if(previous.fingerprint!==fingerprint)throw new Error('Token belongs to another import');return structuredClone(previous.value);}
      const row=value.rows.find(row=>row.agent===action.agent&&row.native_id===action.native_id);
      if(!row||row.intent!==action.intent)throw new Error('Session is no longer importable');
      const id=(action.intent==='fork'?'gui':'observe')+'/lab-import-'+managed.size;
      managed.set(fingerprint,{home,id,agent:row.agent,intent:action.intent,title:row.title+(action.intent==='fork'?' · 分支':'')});
      if(action.intent==='import')value.rows=value.rows.filter(candidate=>candidate!==row);
      const result:SessionImportReply={kind:'imported',home,id,intent:action.intent as ImportIntent};
      receipts.set(action.token,{fingerprint,value:result});return result;
    }
  };
}
