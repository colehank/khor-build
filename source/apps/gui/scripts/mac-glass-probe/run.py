"""Remote native diagnostic. Requires macOS and a verified release Lab artifact."""
import pathlib,subprocess,sys
root=pathlib.Path(__file__).resolve().parents[4]
artifact=pathlib.Path(sys.argv[1]).resolve()
output=pathlib.Path(sys.argv[2]).resolve();output.mkdir(parents=True,exist_ok=True)
css=list(artifact.rglob('assets/main-*.css'))
assert len(css)==1,css
# Same .glass, .glass-sel and transparent window plane as the rc.17 app.
html='''<!doctype html><html data-window-transparent style="--window-opacity:20%"><head><meta charset="utf-8"><style>'''+css[0].read_text()+'''</style><style>
html,body{margin:0;width:600px;height:420px;background:transparent!important}
#root{height:100%;position:relative} .probe{position:absolute;left:40px;width:500px;height:70px;border-radius:35px}
.glass.probe{top:20px}.control{top:120px;background:var(--glass)}
[data-band].probe{top:220px}.glass-sel{position:absolute;left:200px;top:0;width:240px;height:70px;border-radius:35px}
[data-probe-native-backdrop] .glass,[data-probe-native-backdrop] [data-band]::before{-webkit-backdrop-filter:none!important;backdrop-filter:none!important}
</style></head><body><div id="root"><div class="glass probe"><div class="glass-sel"></div></div><div class="control probe"></div><div class="probe" data-band data-scrolled></div></div></body></html>'''
(output/'probe.html').write_text(html)
subprocess.run(['swiftc','-parse-as-library',str(pathlib.Path(__file__).with_name('Probe.swift')),'-o',str(output/'probe'),'-framework','AppKit','-framework','WebKit'],check=True)
subprocess.run([str(output/'probe'),str(output/'probe.html'),str(output)],timeout=90,check=True)
print((output/'result.json').read_text())
