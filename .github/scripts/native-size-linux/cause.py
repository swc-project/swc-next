import hashlib,json,os,pathlib,re,shutil,subprocess
root=pathlib.Path.cwd();out=pathlib.Path(os.environ['RUNNER_TEMP'])/'native-cause';out.mkdir(exist_ok=True)
variant=os.environ['CAUSE_VARIANT'];flags=['-Zlocation-detail=none'] if variant in ['location','both'] else [];extra=['-Zbuild-std=panic_abort,std'] if variant in ['std','both'] else []
env={k:v for k,v in os.environ.items() if not k.startswith('CARGO_PROFILE_') and k not in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_BOOTSTRAP']};env.update(CARGO_ENCODED_RUSTFLAGS='\x1f'.join(flags),CARGO_TARGET_DIR=str(out/'target'),CARGO_PROFILE_RELEASE_STRIP='none')
cmd=['cargo','build','--locked','--release','--target','x86_64-unknown-linux-gnu','-p','swc_next_core_bridge',*extra]
with (out/'build.log').open('w') as log:subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
binary=out/'core.node';shutil.copy2(out/'target/x86_64-unknown-linux-gnu/release/libswc_next_core_bridge.so',binary)
for command,file in [(['nm','-anC',str(binary)],'symbols.txt'),(['objdump','-d','-C',str(binary)],'disassembly.txt')]:
 with (out/file).open('w') as f:subprocess.run(command,stdout=f,check=True)
text=out/'text.bin';subprocess.run(['objcopy','--dump-section',f'.text={text}',str(binary)],check=True)
meta={'variant':variant,'flags':flags,'extra':extra,'text_sha256':hashlib.sha256(text.read_bytes()).hexdigest(),'results':[]}
if variant in ['base','both']:
 side='base' if variant=='base' else 'candidate';old=pathlib.Path(os.environ['RUNNER_TEMP'])/'native-size-linux'/side/'swc-next-core.linux-x64-gnu.node';oldtext=out/'old-text.bin'
 subprocess.run(['objcopy','--dump-section',f'.text={oldtext}',str(old)],check=True)
 meta['matches_original_text']=oldtext.read_bytes()==text.read_bytes()
node=pathlib.Path(subprocess.check_output(['which','node'],text=True).strip());addon=out/'measure.node';subprocess.run(['cc','-shared','-fPIC','-O2',f'-I{node.parent.parent}/include/node','-o',str(addon),'tasks/benchmark/js/measure.c'],check=True)
for fixture in ['astro-config.ts','cal.com.tsx','backbone-1.1.0.js']:
 profile=out/f'{fixture}.callgrind';runenv={**os.environ,'NAPI_RS_NATIVE_LIBRARY_PATH':str(binary),'MEASURE_ADDON':str(addon)}
 cmd=['valgrind','--tool=callgrind','--instr-atstart=no','--collect-atstart=no','--cache-sim=no','--branch-sim=no','--smc-check=all-non-file','--dump-instr=yes','--collect-jumps=yes',f'--callgrind-out-file={profile}',str(node),'--jitless','--predictable','--expose-gc','--random-seed=42','.github/scripts/native-size-linux/profile.mjs','parse',fixture]
 with (out/f'{fixture}.log').open('w') as log:r=subprocess.run(cmd,env=runenv,stdout=subprocess.PIPE,stderr=log,text=True,check=True)
 count=int(re.search(r'^totals:\s+(\d+)',profile.read_text(),re.M)[1]);assert count>0
 meta['results'].append({'fixture':fixture,'instructions':count,'output':json.loads(r.stdout)})
 (out/'cause.json').write_text(json.dumps(meta,indent=2));print(variant,fixture,count,flush=True)
