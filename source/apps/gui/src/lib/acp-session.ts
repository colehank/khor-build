/** ACP v1 option shapes. IDs and choices are owned by the agent. */
export type SessionConfigValue={value:string;name:string;description?:string|null};
export type SessionConfigGroup={group:string;name:string;options:SessionConfigValue[]};
export type SessionConfigOption={id:string;name:string;description?:string|null;category?:string|null;type:string;currentValue?:string|boolean;options?:(SessionConfigValue|SessionConfigGroup)[]};
export type AgentProtocol={agentInfo?:{name:string;title?:string;version?:string};capabilities?:{sessionCapabilities?:Record<string,unknown>;auth?:{logout?:unknown}};authMethods?:{id:string;name:string;type?:string;description?:string}[]};

/** Protocol suffixes describe the connection, not the agent's identity.
 * The original registration remains the launch/routing identifier. */
export function agentPresentation(agent:string){
  const plain=agent.trim().replace(/(?:[-_\s]+acp|\s*[（(]ACP(?:\s*(?:扩展|adapter|extension))?[）)])$/i,'').trim()||agent;
  const key=plain.toLowerCase();
  const name=['codex','codex-cli','codex cli'].includes(key)?'Codex':
    ['claude','claude-agent','claude agent','claude-code','claude code'].includes(key)?'Claude':plain;
  return {name,native:agent==='codex'||agent==='claude'};
}

/** One choice per agent. An already remembered connection stays selected;
 * new choices use the built-in connection where one exists. */
export function conversationAgents(registered:readonly string[],preferred?:string):string[]{
  const names=[...new Set(['claude','codex',...registered])];
  const groups=new Map<string,string>();
  for(const name of names){
    const key=agentPresentation(name).name.toLowerCase();
    if(!groups.has(key)||name===preferred)groups.set(key,name);
  }
  return [...groups.values()];
}

export function conversationRecents(recent:readonly string[],agents:readonly string[]):string[]{
  return [...new Set(recent.map(id=>{
    if(!id.startsWith('agent:'))return id;
    const label=agentPresentation(id.slice(6)).name.toLowerCase();
    const selected=agents.find(name=>agentPresentation(name).name.toLowerCase()===label);
    return selected?'agent:'+selected:id;
  }))];
}

export function agentSetupFailure(reason:unknown,error:unknown):string{
  switch(reason){
    case 'npm_cache_unavailable':return '此机器的 npm 缓存不可用，修复后重试';
    case 'npm_package_unavailable':return '找不到此版本的智能体扩展';
    case 'npm_download_failed':return '智能体扩展下载失败，检查此机器的网络后重试';
    case 'node_unavailable':return '此机器未找到 Node.js，安装后重试';
  }
  if(typeof error==='string'&&/incoming[_ ]transport[_ ]closed/i.test(error))return '未能连接智能体，请重试';
  return typeof error==='string'&&error?error:'无法读取智能体能力';
}

export function configOptions(value:unknown):SessionConfigOption[]{
  return Array.isArray(value)?value.filter((v):v is SessionConfigOption=>v&&typeof v==='object'&&typeof v.id==='string'&&typeof v.name==='string'&&(v.type==='boolean'?typeof v.currentValue==='boolean':v.type==='select'&&typeof v.currentValue==='string'&&Array.isArray(v.options))):[];
}
