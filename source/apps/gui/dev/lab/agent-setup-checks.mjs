import assert from 'node:assert/strict';

export async function checkAgentSetup(page,mobile,capture){
  if(mobile){
    const band=page.locator('[data-session-screen] > [data-session-bar] [data-deck-band]:visible').first();
    const title=band.locator('button[data-deck-title]');
    await (await title.count()?title:band).press('ArrowUp');
    await page.locator('[data-tab-sheet-new]:visible').click();
  }
  else await page.locator('[data-session-tabs] [data-tab-new]:visible').first().click();
  const body=page.locator('[data-new-session-page]:visible');await body.waitFor();
  const launch=body.locator('[data-new-session-launch] textarea');
  const select=async label=>{await launch.click();await page.getByRole('option').filter({has:page.getByText(label,{exact:true})}).click();};
  const read=async()=>{
    const setup=body.locator('[data-acp-setup]');await setup.waitFor();
    if(!await setup.evaluate(el=>el.open))await setup.locator('summary').click();
    await setup.getByRole('button',{name:'读取智能体能力',exact:true}).click();
    return setup;
  };
  const machine=await body.locator('[data-new-session-machines]').getAttribute('data-machine');
  const directory=body.locator('[data-new-session-dir]');
  const original=await directory.inputValue();
  await directory.fill('/preview/new-session-directory/nested');
  await body.locator('[data-new-session-directory-missing]').waitFor();
  assert.equal(await body.locator('[data-new-session-directory-missing]').innerText(),'此目录不存在，创建会话时将新建');
  const parent=await page.evaluate(async(machine)=>(await(await fetch('/api/ls',{method:'POST',body:JSON.stringify({machine,path:'/preview'})})).json()),machine);
  assert(!parent.entries.some(entry=>entry.name==='new-session-directory'),'Typing a new directory must not create it');
  await capture('missing-directory');
  await directory.fill(original);
  await body.locator('[data-new-session-directory-missing]').waitFor({state:'detached'});
  await launch.click();
  for(const name of ['Codex','Claude','DeepSeek Harness'])assert.equal(await page.getByRole('option').filter({has:page.getByText(name,{exact:true})}).count(),1,'One GUI entry for '+name);
  assert(!(await page.locator('[data-choice-option^="agent:"]').allTextContents()).some(label=>/acp/i.test(label)),'Protocol suffixes never appear in agent choices');
  await page.locator('[data-choice-option="agent:codex"]').click();
  for(const name of ['Codex','Claude']){
    await select(name);const setup=await read();
    await setup.locator('[data-acp-setup-ready]').waitFor();
    assert.match(await setup.innerText(),new RegExp('已连接 '+name));
    assert.match(await setup.innerText(),new RegExp('登录由此机器上的 '+name+' 管理'));
    assert.equal(await setup.locator('[role=alert]').count(),0);
  }
  await select('reviewer');const other=await read();await other.locator('[data-acp-setup-ready]').waitFor();
  assert.equal((await other.locator('[data-acp-setup-ready]').innerText()).trim(),'已连接 reviewer','Extension-reported names also omit the protocol suffix');
  await select('DeepSeek Harness');const dsh=await read();await dsh.locator('[data-acp-setup-ready]').waitFor();
  assert.equal((await dsh.locator('[data-acp-setup-ready]').innerText()).trim(),'已连接 DeepSeek Harness');
  assert.match(await dsh.locator('[data-dsh-credentials]').innerText(),/所选电脑/);
  assert.equal(await dsh.getByText('附加工作目录（每行一个，位于执行机器）',{exact:true}).count(),0);
  assert.equal(await body.locator('[data-new-session-launch] [data-brand-mark="dsh"]').count(),1);
  await capture('dsh-setup');
  await select('Codex · 终端');assert.equal(await body.locator('[data-acp-setup]').count(),0,'Native terminal never sends ACP setup');
  await select('Codex');
  await page.evaluate(()=>fetch('/api/__ai_preview/acp-setup',{method:'POST',body:JSON.stringify({fail:true})}));
  let setup=await read();await setup.locator('[role=alert]').waitFor();
  assert.equal(await setup.locator('[role=alert]').innerText(),'此机器的 npm 缓存不可用，修复后重试');
  await capture('setup-failure');
  await page.evaluate(()=>fetch('/api/__ai_preview/acp-setup',{method:'POST',body:JSON.stringify({fail:false})}));
  setup=await read();await setup.locator('[data-acp-setup-ready]').waitFor();
  assert.equal(await setup.locator('[role=alert]').count(),0,'Retry clears the prior error');
  const calls=await page.evaluate(async()=>await(await fetch('/api/__ai_preview/acp-setup-calls',{method:'POST',body:'{}'})).json());
  assert.deepEqual(calls.map(call=>call.agent),['codex','claude','reviewer-acp','dsh','codex','codex']);
  assert(calls.every(call=>call.machine===machine&&call.operation==='acp_initialize'),'Setup follows the selected execution machine');
  assert(await body.evaluate(el=>el.scrollWidth<=el.clientWidth),'The actual new session page must fit its viewport');
  await capture('setup-ready');
}
