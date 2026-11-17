import assert from 'node:assert/strict';
import {agentPresentation,conversationAgents,conversationRecents,agentSetupFailure,configValueName} from '../src/lib/acp-session.ts';

assert.equal(agentPresentation('codex').name,'Codex');
assert.equal(agentPresentation('claude').native,true);
assert.equal(agentPresentation('codex-acp').name,'Codex');
assert.equal(agentPresentation('claude-acp').native,false);
assert.equal(agentPresentation('custom-acp').name,'custom');
assert.equal(agentPresentation('Claude Agent (ACP)').name,'Claude');
assert.equal(agentPresentation(' Codex (ACP) ').name,'Codex');
assert.equal(agentPresentation('reviewer_acp').name,'reviewer');
assert.equal(agentPresentation('acp-research').name,'acp-research','Only the protocol suffix is removed');
assert.equal(agentPresentation('my-acp-agent').name,'my-acp-agent');
for(const name of ['dsh','deepseek-harness','deepseek-harness-acp'])assert.equal(agentPresentation(name).name,'DeepSeek Harness');
const registered=['codex-acp','claude-acp','reviewer-acp','reviewer'];
assert.deepEqual(conversationAgents(registered),['claude','codex','dsh','reviewer-acp'],'One visible choice per agent');
const restored=conversationAgents(registered,'codex-acp');
assert.deepEqual(restored,['claude','codex-acp','dsh','reviewer-acp'],'Remembered connections retain their exact routing identifier');
assert.deepEqual(conversationAgents(['deepseek-harness-acp','dsh']),['claude','codex','dsh']);
assert.deepEqual(conversationAgents(['deepseek-harness-acp'],'deepseek-harness-acp'),['claude','codex','deepseek-harness-acp']);
assert.deepEqual(conversationRecents(['agent:codex','term:claude','agent:codex-acp','agent:reviewer'],restored),['agent:codex-acp','term:claude','agent:reviewer-acp'],'Grouped choices keep recent order without changing saved connections');
for(const reason of ['npm_cache_unavailable','npm_package_unavailable','npm_download_failed','node_unavailable']){
  const message=agentSetupFailure(reason,'transport closed token=private');
  assert.ok(message&&!message.includes('private'));
}
assert.equal(agentSetupFailure(null,'Error: Incoming transport closed: {"method":"initialize"}'),'未能连接智能体，请重试');
assert.equal(agentSetupFailure(null,'Permission denied'),'Permission denied');
assert.equal(agentSetupFailure(null,null),'无法读取智能体能力');
const opaque='["deepseek-official","deepseek-flash"]';
const model={id:'model',name:'Model',type:'select',currentValue:opaque,options:[{group:'deepseek-official',name:'DeepSeek',options:[{value:opaque,name:'DeepSeek-V4.1-Flash'}]}]};
assert.equal(configValueName(model),'DeepSeek-V4.1-Flash');
assert.equal(model.currentValue,opaque,'Presentation must not rewrite the routing value');
assert.equal(configValueName({...model,currentValue:'unlisted'}),'unlisted');
console.log('Agent setup: protocol-free names, single choices, preserved routing and recents, finite startup causes and private stderr isolation passed');
