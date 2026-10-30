import hashlib,json,os,pathlib,re,subprocess
root=pathlib.Path.cwd();inputs=pathlib.Path(os.environ['RUNNER_TEMP'])/'native-size-linux';out=pathlib.Path(os.environ['RUNNER_TEMP'])/'native-instructions';out.mkdir(exist_ok=True)
operation=os.environ['PROFILE_OPERATION'];fixture=os.environ['PROFILE_FIXTURE']
metadata={'operation':operation,'fixture':fixture,'node':subprocess.check_output(['node','--version'],text=True),'cpu':subprocess.check_output(['lscpu'],text=True),'valgrind':subprocess.check_output(['valgrind','--version'],text=True),'mode':'jitless; native NAPI parse/print only; encode/decode/setup excluded; 10 warmups; 3 measured calls; 3 fresh trials each; AB/BA order','results':[]}
node=pathlib.Path(subprocess.check_output(['which','node'],text=True).strip());addon=out/'measure.node'
subprocess.run(['cc','-shared','-fPIC','-O2',f'-I{node.parent.parent}/include/node','-o',str(addon),'tasks/benchmark/js/measure.c'],check=True)
records=json.loads((inputs/'sizes.json').read_text())
for row in records:
 binary=inputs/row['name']/'swc-next-core.linux-x64-gnu.node'
 assert hashlib.sha256(binary.read_bytes()).hexdigest()==row['artifacts']['core']['sha256']
for trial in range(3):
 for side in (['base','candidate'] if trial%2==0 else ['candidate','base']):
  name=f'{side}-{trial}';profile=out/f'{name}.callgrind';env={**os.environ,'NAPI_RS_NATIVE_LIBRARY_PATH':str(inputs/side/'swc-next-core.linux-x64-gnu.node'),'MEASURE_ADDON':str(addon)}
  cmd=['valgrind','--tool=callgrind','--instr-atstart=no','--collect-atstart=no','--cache-sim=no','--branch-sim=no','--smc-check=all-non-file',f'--callgrind-out-file={profile}',str(node),'--jitless','--predictable','--expose-gc','--random-seed=42','.github/scripts/native-size-linux/profile.mjs',operation,fixture]
  with (out/f'{name}.log').open('w') as log:
   result=subprocess.run(cmd,env=env,stdout=subprocess.PIPE,stderr=log,text=True,check=True)
  payload=json.loads(result.stdout);summary=re.search(r'^totals:\s+(\d+)',profile.read_text(),re.M);assert summary and int(summary[1]) > 0
  metadata['results'].append({'side':side,'trial':trial,'instructions':int(summary[1]),'output':payload,'command':cmd})
  (out/'instructions.json').write_text(json.dumps(metadata,indent=2))
  print(side,trial,summary[1],flush=True)
assert len(set(x['output']['output'] for x in metadata['results']))==1,'Output changed between builds'
