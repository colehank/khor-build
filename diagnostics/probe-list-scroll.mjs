import {chromium} from 'playwright-core';
import {spawn} from 'node:child_process';
import {mkdir,writeFile} from 'node:fs/promises';
import path from 'node:path';
const report=process.env.LAB_REPORT;
await mkdir(report,{recursive:true});
const server=spawn('python3',['-m','khor_lab.preview',process.env.LAB_BUNDLE,'--port','23918'],{
  cwd:path.resolve('../..'),env:{...process.env,PYTHONPATH:'scripts'},stdio:'inherit',
});
const effects={
  normal:'',
  'content-mask-off':'html :is([data-sticky-fading],[data-sticky-surface]){mask-image:none!important;-webkit-mask-image:none!important}',
  'header-blur-off':'html :is([data-band],[data-sticky-head])::before{backdrop-filter:none!important;-webkit-backdrop-filter:none!important}',
};
const results=[];
const frames=page=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
let browser;
try{
  for(let i=0;i<100;i++){
    try{if((await fetch('http://127.0.0.1:23918/lab/status')).ok)break;}catch{}
    await new Promise(resolve=>setTimeout(resolve,100));
  }
  browser=await chromium.launch({headless:true});
  for(const [mode,css] of Object.entries(effects)){
    const context=await browser.newContext({viewport:{width:390,height:844},isMobile:true,hasTouch:true});
    const page=await context.newPage(),entry={mode,samples:[],nativeTrace:[]};
    try{
      await page.route('**/lab/runtime*',route=>route.fulfill({contentType:'application/json',body:JSON.stringify({mode:'frontend',scene:'workspaces',tree:'fade-probe',revision:'one'})}));
      await page.goto('http://127.0.0.1:23918/app/?scene=workspaces&machines=6&theme=dark');
      await page.locator('[data-ws-row]').first().waitFor();
      if(css)await page.addStyleTag({content:css});
      await page.waitForTimeout(800);
      const port=page.locator('[data-list] [data-pane-body]').first();
      const facts=()=>port.evaluate(el=>{
        const surface=el.parentElement,style=getComputedStyle(el),visual=getComputedStyle(surface);
        return {top:el.scrollTop,nativeMask:style.maskImage||style.webkitMaskImage,
          visual:surface.hasAttribute('data-scroll-viewport')&&surface.hasAttribute('data-sticky-surface'),
          visualMask:visual.maskImage||visual.webkitMaskImage,height:el.clientHeight};
      });
      for(const top of [0,1,3,52,0]){
        await port.evaluate((el,top)=>{el.scrollTop=top;},top);await frames(page);entry.samples.push(await facts());
      }
      const cdp=await context.newCDPSession(page);
      try{
        for(const [type,y] of [['touchStart',280],...[260,230,200,170,200,230,260,290,320,290,260,230,200,170].map(y=>['touchMove',y]),['touchEnd',170]]){
          await cdp.send('Input.dispatchTouchEvent',{type,touchPoints:type==='touchEnd'?[]:[{x:200,y}]});
          await page.waitForTimeout(30);entry.nativeTrace.push(await facts());
        }
      }finally{await cdp.detach();}
      await page.waitForTimeout(500);
      await port.evaluate(el=>{el.scrollTop=52;});await frames(page);
      await port.evaluate(el=>{
        const target=el.parentElement.hasAttribute('data-scroll-viewport')?el.parentElement:el;
        window.fadeChanges=[];
        window.fadeObserver=new MutationObserver(records=>window.fadeChanges.push(...records.map(r=>({name:r.attributeName,old:r.oldValue}))));
        window.fadeObserver.observe(target,{attributes:true,attributeOldValue:true,attributeFilter:['style','data-sticky-fading','data-sticky-surface']});
        el.querySelector('[data-side-title]').appendChild(document.createTextNode(''));
      });
      await frames(page);
      entry.changes=await page.evaluate(()=>{window.fadeObserver.disconnect();return window.fadeChanges;});
      await page.screenshot({path:path.join(report,mode+'.png')});
    }catch(error){entry.error=String(error);}
    finally{results.push(entry);await writeFile(path.join(report,'probe.json'),JSON.stringify(results,null,2));await context.close();}
  }
  const normal=results.find(row=>row.mode==='normal');
  const errors=[];
  if(normal.error)errors.push(normal.error);
  if(normal.samples.some(row=>row.nativeMask!=='none'))errors.push('The native scroll port itself gains a mask at fade onset');
  if(normal.samples.some(row=>!row.visual||row.visualMask==='none'))errors.push('A separate persistent visual surface is missing');
  if(normal.changes?.length)errors.push('A content update tears down or rewrites an unchanged fade');
  await writeFile(path.join(report,'verdict.json'),JSON.stringify({errors,physicalIphone:'user A/B confirms content-mask-off is smoother; this probe does not certify device frame timing'},null,2));
  console.log(JSON.stringify({errors}));
  if(errors.length)process.exitCode=1;
}finally{await browser?.close();server.kill('SIGTERM');}
