import type {ImportIntent} from '@/api';

// A caller retry record, not an execution ledger. IndexedDB serializes claims
// across windows; only the source decides whether this token has taken effect.
const DATABASE='khor-session-import', STORE='pending', LIMIT=64;
type Pending={token:string;intent:ImportIntent};
function open():Promise<IDBDatabase>{
  return new Promise((resolve,reject)=>{
    const request=indexedDB.open(DATABASE,1);
    request.onupgradeneeded=()=>request.result.createObjectStore(STORE);
    request.onerror=()=>reject(new Error('Cannot preserve the import retry; no action was sent'));
    request.onblocked=()=>reject(new Error('Import retry storage is blocked; no action was sent'));
    request.onsuccess=()=>{request.result.onversionchange=()=>request.result.close();resolve(request.result);};
  });
}
async function edit(home:string,agent:string,native:string,intent:ImportIntent,settled?:string):Promise<string>{
  if(!home||!agent||!native||home.length>256||agent.length>256||native.length>4096)throw new Error('Invalid import identity');
  const db=await open(),key=[home,agent,native];
  try{return await new Promise((resolve,reject)=>{
    const tx=db.transaction(STORE,'readwrite'),store=tx.objectStore(STORE),read=store.get(key);
    let answer='',failure:unknown;
    read.onsuccess=()=>{try{
      const old=read.result as Pending|undefined;
      if(old&&(!/^[0-9a-f-]{36}$/.test(old.token)||(old.intent!=='import'&&old.intent!=='fork')))throw new Error('Invalid saved import retry');
      if(settled){if(old?.token===settled&&old.intent===intent)store.delete(key);answer=settled;return;}
      if(old){if(old.intent!==intent)throw new Error('A previous import is unresolved; check the session list before changing the action');answer=old.token;return;}
      answer=crypto.randomUUID();
      const count=store.count();count.onsuccess=()=>{if(count.result>=LIMIT){failure=new Error('Too many unresolved imports; check the session list');tx.abort();}else store.put({token:answer,intent} satisfies Pending,key);};
    }catch(error){failure=error;tx.abort();}};
    tx.oncomplete=()=>resolve(answer);
    tx.onabort=()=>reject(failure??new Error('Cannot preserve the import retry; no action was sent'));
    tx.onerror=()=>{failure??=new Error('Cannot preserve the import retry; no action was sent');};
  });}finally{db.close();}
}
export const claimImportIntent=(home:string,agent:string,native:string,intent:ImportIntent)=>edit(home,agent,native,intent);
export const settleImportIntent=(home:string,agent:string,native:string,intent:ImportIntent,token:string)=>edit(home,agent,native,intent,token);
