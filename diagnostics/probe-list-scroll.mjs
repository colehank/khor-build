import {chromium,webkit} from 'playwright-core';
import {spawn} from 'node:child_process';
import {mkdir,writeFile} from 'node:fs/promises';
import path from 'node:path';
const report=process.env.LAB_REPORT;await mkdir(report,{recursive:true});
const server=spawn('python3',['-m','khor_lab.preview',process.env.LAB_BUNDLE,'--port','23918'],{cwd:path.resolve('../..'),env:{...process.env,PYTHONPATH:'scripts'},stdio:'inherit'});
const results=[];
const frames=page=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
async function pan(page){const cdp=await page.context().newCDPSession(page),trace=[];try{for(const [type,y] of [['touchStart',280],...[300,330,360,390,420,390,360,330,300,270,240,210,180].map(y=>['touchMove',y]),['touchEnd',180]]){await cdp.send('Input.dispatchTouchEvent',{type,touchPoints:type==='touchEnd'?[]:[{x:200,y}]});await page.waitForTimeout(30);trace.push(await page.locator('[data-pane-body]').evaluate(el=>el.scrollTop));}await cdp.send('Input.synthesizeScrollGesture',{x:200,y:280,yDistance:-150,gestureSourceType:'touch',preventFling:true});return {trace,synthesized:await page.locator('[data-pane-body]').evaluate(el=>el.scrollTop)};}finally{await cdp.detach();}}
try{
 for(let i=0;i<100;i++){try{if((await fetch('http://127.0.0.1:23918/lab/status')).ok)break;}catch{}await new Promise(r=>setTimeout(r,100));}
 for(const [name,engine] of [['chromium',chromium],['webkit',webkit]]){
  const browser=await engine.launch({headless:true}),context=await browser.newContext({viewport:{width:390,height:844},isMobile:true,hasTouch:true}),page=await context.newPage();
  const row={engine:name};try{
   if(name==='chromium'){
    await page.setContent('<meta name="viewport" content="width=device-width,initial-scale=1"><div data-pane-body style="height:700px;overflow:auto"><div style="height:3000px">Control</div></div>');await page.locator('[data-pane-body]').evaluate(el=>{el.scrollTop=220;});row.control=await pan(page);
   }
   await page.route('**/lab/runtime*',route=>route.fulfill({contentType:'application/json',body:JSON.stringify({mode:'frontend',scene:'workspaces',tree:'probe',revision:'one'})}));
   await page.goto('http://127.0.0.1:23918/app/?scene=workspaces&machines=6&theme=dark');await page.locator('[data-ws-row]').first().waitFor();await page.waitForTimeout(800);
   await page.evaluate(()=>{
    window.probeEvents=[];const prevent=Event.prototype.preventDefault;Event.prototype.preventDefault=function(){if(this.type.startsWith('touch'))window.probeEvents.push({type:this.type,cancelable:this.cancelable,trusted:this.isTrusted,stack:new Error().stack?.slice(0,1600)});return prevent.call(this);};
    window.probeHit=[];for(let el=document.elementFromPoint(200,280);el;el=el.parentElement)window.probeHit.push({tag:el.tagName,attrs:[...el.attributes].filter(a=>a.name.startsWith('data-')).map(a=>a.name),touch:getComputedStyle(el).touchAction,overflow:getComputedStyle(el).overflowY});
   });
   if(name==='chromium'){await page.locator('[data-pane-body]').evaluate(el=>{el.scrollTop=220;});row.appPan=await pan(page);}
   row.hit=await page.evaluate(()=>window.probeHit);row.events=await page.evaluate(()=>window.probeEvents);
   const port=page.locator('[data-list] [data-pane-body]').first();await port.evaluate(el=>{el.scrollTop=0;});await frames(page);
   const facts=()=>port.evaluate(el=>({owner:el.hasAttribute('data-band-scroll'),height:el.clientHeight,content:el.scrollHeight,padding:getComputedStyle(el).paddingTop,top:el.scrollTop,pull:!!el.closest('[data-pull-host]')}));row.rest=await facts();
   const touch=(type,y)=>port.evaluate((el,{type,y})=>{const p={identifier:1,target:el,clientX:200,clientY:y};const e=new Event(type,{bubbles:true,cancelable:true});Object.defineProperties(e,{touches:{value:type==='touchcancel'?[]:[p]},changedTouches:{value:[p]}});el.dispatchEvent(e);},{type,y});
   await touch('touchstart',250);await touch('touchmove',330);await frames(page);row.pulling=await facts();await port.evaluate(el=>el.querySelector('[data-side-title]').appendChild(document.createTextNode('')));await frames(page);row.afterUpdate=await facts();await page.screenshot({path:path.join(report,name+'-pull.png')});await touch('touchcancel',330);await frames(page);row.cancelled=await facts();
  }catch(error){row.error=String(error);}finally{results.push(row);await writeFile(path.join(report,'probe.json'),JSON.stringify(results,null,2));await browser.close();}
 }
}finally{server.kill('SIGTERM');}
