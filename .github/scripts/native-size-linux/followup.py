import hashlib,json,os,pathlib,subprocess
out=pathlib.Path(os.environ['RUNNER_TEMP'])/'native-size-linux'
metadata={'cpu':subprocess.check_output(['lscpu'],text=True),'node':subprocess.check_output(['node','--version'],text=True),'harness_revision':subprocess.check_output(['git','rev-parse','HEAD'],text=True),'original_run':40995106402,'gc':'explicit full GC outside each measured round'}
records=json.loads((out/'sizes.json').read_text())
for row in records:
 for c,a in row['artifacts'].items():
  assert hashlib.sha256((out/row['name']/f'swc-next-{c}.linux-x64-gnu.node').read_bytes()).hexdigest()==a['sha256']
cpu=min(os.sched_getaffinity(0));os.sched_setaffinity(0,{cpu});metadata['benchmark_cpu']=cpu
(out/'followup-metadata.json').write_text(json.dumps(metadata,indent=2))
bench=['node','.github/scripts/native-size-linux/bench.mjs'];base=str(out/'base/swc-next-core.linux-x64-gnu.node');candidate=str(out/'candidate/swc-next-core.linux-x64-gnu.node')
for kind,a,b in [('control-before',base,base),('comparison',base,candidate),('control-after',candidate,candidate)]:
 with (out/f'followup-{kind}.log').open('w') as log:subprocess.run([*bench,a,b,str(out/f'followup-{kind}.json'),kind],stdout=log,stderr=subprocess.STDOUT,check=True)
 print(kind,'done',flush=True)
