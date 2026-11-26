// Production ended reader with controlled transport; native wire routing has
// separate node tests. No shell or user store is opened by this browser check.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdtempSync,readFileSync,writeFileSync,rmSync,mkdirSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createServer} from 'vite';
import {chromium} from 'playwright-core';

const gui=fileURLToPath(new URL('..',import.meta.url));
const baseline=process.env.ENDED_TERMINAL_BASELINE;
const scratch=mkdtempSync(join(gui,'.ended-check-'));
const output=process.env.ENDED_TERMINAL_OUT||mkdtempSync(join(tmpdir(),'khor-ended-terminal-'));
mkdirSync(output,{recursive:true});
const sources=['src/views/TerminalPane.tsx','src/views/DetailPane.tsx','src/api.ts'];
const fingerprint=()=>Object.fromEntries(sources.map(name=>[name,createHash('sha256').update(readFileSync(join(gui,name))).digest('hex')]));
const before=fingerprint(),errors=[],calls=[],checks=[];
let vite,browser,context,page,mode={kind:'hold'},held=[],closing=false;
const ledger={ended:{exit:7},screen:null,startup_error:null,recent:[{text:'inspect result',cwd:'/work/中文 project',exit:7,started_ms:1,ended_ms:2}]};
const screen={ended:{exit:0},recent:[],startup_error:null,screen:{cols:80,rows:24,at_ms:1,bytes:Buffer.from('SAVED REMOTE OUTPUT\r\n').toString('base64')}};
writeFileSync(join(scratch,'probe.tsx'),`
import React,{useState} from 'react';import {createRoot} from 'react-dom/client';
import {EndedPane} from '/src/views/TerminalPane';import {TooltipProvider} from '/src/components/ui/tooltip';
import {gui} from '/src/gen/catalog';import '/src/styles/tokens.css';import '/src/styles/app.css';
function Probe(){const [row,setRow]=useState({id:'shell/initial',at:1});window.__endedProbe={words:gui,select:(id)=>setRow(w=>({id,at:w.at+1})),refresh:()=>setRow(w=>({...w,at:w.at+1}))};return <TooltipProvider><div style={{height:'100vh',display:'flex',flexDirection:'column',minHeight:0}}><EndedPane id={row.id} at={row.at}/></div></TooltipProvider>;}
createRoot(document.getElementById('root')).render(<Probe/>);
`);
writeFileSync(join(scratch,'index.html'),'<html data-theme="light"><div id="root"></div><script type="module" src="./probe.tsx"></script></html>');
const record=name=>{checks.push(name);console.log('PASS',name);};
const setMode=(kind,value=null)=>{mode={kind,value};};
const select=async(id)=>{
  const count=calls.length;
  await page.evaluate(id=>window.__endedProbe.select(id),id);
  await page.waitForFunction(count=>window.__endedCalls>count,count);
};
const release=async(kind,value)=>{
  const requests=held;held=[];
  for(const request of requests)request({kind,value});
};
try{
  const plugins=baseline?[{name:'ended-published-baseline',enforce:'pre',load(id){
    if(id.split('?')[0]===join(gui,'src/views/TerminalPane.tsx'))return execFileSync('git',['show',`${baseline}:apps/gui/src/views/TerminalPane.tsx`],{cwd:gui,encoding:'utf8'});
  }}]:[];
  vite=await createServer({configFile:join(gui,'vite.config.ts'),root:gui,cacheDir:join(scratch,'cache'),plugins,
    optimizeDeps:{entries:[join(scratch,'probe.tsx')]},server:{host:'127.0.0.1',port:0,hmr:false}});
  await vite.listen();const address=vite.httpServer.address();assert(address&&typeof address==='object');const port=address.port;
  browser=await chromium.launch({channel:process.env.PLAYWRIGHT_CHROMIUM_CHANNEL===undefined?'chrome':process.env.PLAYWRIGHT_CHROMIUM_CHANNEL||undefined,headless:true,args:['--no-proxy-server']});
  context=await browser.newContext({viewport:{width:900,height:650},reducedMotion:'reduce'});
  await context.route(`http://127.0.0.1:${port}/**`,async route=>{
    const headers={'access-control-allow-origin':`http://localhost:${port}`,'access-control-allow-methods':'POST, OPTIONS','access-control-allow-headers':'content-type'};
    if(route.request().method()==='OPTIONS')return route.fulfill({status:204,headers});
    const command=new URL(route.request().url()).pathname.slice(1);
    if(command!=='term_ended')return route.fulfill({headers,contentType:'application/json',body:'null'});
    const args=route.request().postDataJSON();calls.push(args);
    await route.request().frame().evaluate(count=>window.__endedCalls=count,calls.length);
    const answer=mode.kind==='hold'?await new Promise(resolve=>held.push(resolve)):mode;
    try{await route.fulfill({status:answer.kind==='error'?500:200,headers,contentType:'application/json',body:JSON.stringify(answer.value)});}
    catch(error){if(!closing)throw error;}
  });
  page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));
  await page.goto(`http://localhost:${port}/${scratch.split('/').at(-1)}/index.html?bridge=${port}`);
  await page.locator('[data-term-ended-wait]').waitFor();
  await page.locator('[data-region-state="loading-never"] [data-skeleton]').first().waitFor({timeout:3000});
  record('initial remote read displays skeleton rather than an empty board');
  await release('error','Remote history is offline');
  await page.locator('[data-region-state="error"]').waitFor();
  assert.match(await page.locator('[data-region-state="error"]').innerText(),/Remote history is offline/);
  setMode('value',null);await page.locator('[data-region-retry]').click();
  await page.locator('[data-region-state="empty"]').waitFor();
  record('read failure retains its reason and retry reaches an explicit empty result');

  setMode('value',ledger);await select('shell/remote-ledger');
  await page.locator('[data-term-ended-row]').waitFor();
  assert.equal(await page.locator('[data-term-ended-command]').innerText(),'inspect result');
  assert.equal(await page.locator('[data-term-ended-exit]').innerText().then(text=>text.includes('7')),true);
  record('remote command ledger preserves command, directory and exit status');

  setMode('hold');await page.evaluate(()=>window.__endedProbe.refresh());
  await page.locator('[data-term-ended][aria-busy="true"]').waitFor();
  assert.equal(await page.locator('[data-term-ended-command]').innerText(),'inspect result');
  await release('error','History refresh failed');
  await page.locator('[data-term-ended-refresh-error]').waitFor();
  assert.equal(await page.locator('[data-term-ended-command]').innerText(),'inspect result');
  setMode('value',screen);await page.locator('[data-term-ended-why] button').click();
  await page.locator('[data-term-ended-drawn]').waitFor();
  const retained=await page.locator('[data-term-ended-screen]').evaluate(box=>{
    const buffer=box.__xterm.buffer.active;
    return Array.from({length:buffer.length},(_,i)=>buffer.getLine(i)?.translateToString(true)).join('\n');
  });
  assert.match(retained,/SAVED REMOTE OUTPUT/);
  record('refresh keeps known content; retry renders the actual saved terminal snapshot');

  setMode('value',{ended:{exit:127},screen:null,recent:[],startup_error:'Selected shell could not be started'});
  await select('shell/startup-failure');await page.locator('[data-term-ended-startup-error]').waitFor();
  assert.equal(await page.locator('[data-term-ended-startup-error]').innerText(),'Selected shell could not be started');
  record('a failed shell startup remains readable when no screen was ever written');

  setMode('hold');await select('shell/late');await page.locator('[data-region-state="loading-never"]').waitFor();
  setMode('value',ledger);await select('shell/current');await page.locator('[data-term-ended-command]').waitFor();
  const lateResponse=page.waitForResponse(response=>response.url().endsWith('/term_ended')&&response.request().postDataJSON()?.id==='shell/late');
  await release('value',screen);
  await (await lateResponse).finished();
  await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
  await page.waitForFunction(()=>document.querySelector('[data-term-ended-command]')?.textContent==='inspect result');
  assert.equal(await page.locator('[data-term-ended-screen]').count(),0);
  record('a late result for a previous session cannot replace the current identity');
  assert.deepEqual(errors,[],'production reader raised a browser error');
  await page.screenshot({path:join(output,'ended-terminal.png')});
}finally{
  closing=true;
  await release('value',null);
  await context?.close();await browser?.close();await vite?.close();
  assert.deepEqual(fingerprint(),before,'the browser check changed production source');
  writeFileSync(join(output,'report.json'),JSON.stringify({checks,errors,calls,sources:before,baseline:baseline??null},null,2));
  rmSync(scratch,{recursive:true,force:true});
}
console.log(`Ended terminal checks: ${checks.length}; evidence ${output}`);
