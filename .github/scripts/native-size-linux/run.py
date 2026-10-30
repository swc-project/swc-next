import gzip,hashlib,json,os,pathlib,shutil,subprocess,time
root=pathlib.Path.cwd();out=pathlib.Path(os.environ['RUNNER_TEMP'])/'native-size-linux';out.mkdir(exist_ok=True)
def capture(args):return subprocess.check_output(args,text=True).strip()
metadata={'revision':capture(['git','rev-parse','HEAD']),'source_revision':'6bab3044','rustc':capture(['rustc','-Vv']),'node':capture(['node','--version']),'cpu':capture(['lscpu']),'os':pathlib.Path('/etc/os-release').read_text(),'uname':capture(['uname','-a']),'lock_sha256':hashlib.sha256((root/'Cargo.lock').read_bytes()).hexdigest()}
(out/'metadata.json').write_text(json.dumps(metadata,indent=2))
records=[]
for name,flags,extra in [('base',[],[]),('candidate',['-Zlocation-detail=none'],['-Zbuild-std=panic_abort,std'])]:
 dest=out/name;dest.mkdir(exist_ok=True)
 env={k:v for k,v in os.environ.items() if not k.startswith('CARGO_PROFILE_') and k not in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_BOOTSTRAP','NAPI_RS_NATIVE_LIBRARY_PATH']}
 env['CARGO_ENCODED_RUSTFLAGS']='\x1f'.join(flags);env['CARGO_TARGET_DIR']=str(out/'target')
 row={'name':name,'flags':flags,'extra':extra,'artifacts':{}}
 for c in ['core','parser','codegen']:
  cmd=['cargo','build','--locked','--release','--target','x86_64-unknown-linux-gnu','-p',f'swc_next_{c}_bridge',*extra]
  print(name,c,'building',flush=True)
  with (dest/f'{c}-build.log').open('w') as log:subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
  binary=dest/f'swc-next-{c}.linux-x64-gnu.node'
  shutil.copy2(out/'target/x86_64-unknown-linux-gnu/release'/f'libswc_next_{c}_bridge.so',binary)
  shutil.copy2(binary,root/'npm'/c/'src-js'/binary.name)
  data=binary.read_bytes();row['artifacts'][c]={'bytes':len(data),'gzip_bytes':len(gzip.compress(data,compresslevel=9,mtime=0)),'sha256':hashlib.sha256(data).hexdigest(),'command':cmd}
  (dest/f'{c}-sections.txt').write_text(capture(['readelf','-SW',str(binary)]))
  print(name,c,row['artifacts'][c],flush=True)
 with (dest/'tests.log').open('w') as log:subprocess.run(['pnpm','test'],env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
 row['tests']='passed';records.append(row);(out/'sizes.json').write_text(json.dumps(records,indent=2))
# No concurrent compilation during timing; inherit a single allowed CPU for all benchmark workers.
cpu=min(os.sched_getaffinity(0));os.sched_setaffinity(0,{cpu});metadata['benchmark_cpu']=cpu;(out/'metadata.json').write_text(json.dumps(metadata,indent=2))
bench=['node','.github/scripts/native-size-linux/bench.mjs'];base=str(out/'base/swc-next-core.linux-x64-gnu.node');candidate=str(out/'candidate/swc-next-core.linux-x64-gnu.node')
for kind,a,b in [('control-before',base,base),('comparison',base,candidate),('control-after',candidate,candidate)]:
 with (out/f'{kind}.log').open('w') as log:subprocess.run([*bench,a,b,str(out/f'{kind}.json'),kind],stdout=log,stderr=subprocess.STDOUT,check=True)
 print(kind,'done',flush=True)
