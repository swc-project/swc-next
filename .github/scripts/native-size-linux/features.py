import gzip, hashlib, json, os, pathlib, platform, shutil, struct, subprocess

root = pathlib.Path.cwd()
out = pathlib.Path(os.environ.get('FEATURE_OUT', str(pathlib.Path(os.environ.get('RUNNER_TEMP', '/tmp')) / 'hashbrown-features')))
out.mkdir(parents=True, exist_ok=True)
target = os.environ.get('FEATURE_TARGET', 'x86_64-unknown-linux-gnu')
is_linux = target == 'x86_64-unknown-linux-gnu'
suffix = 'linux-x64-gnu' if is_linux else 'darwin-arm64'
ext = 'so' if is_linux else 'dylib'
manifest = root / 'Cargo.toml'
candidate = manifest.read_bytes()
baseline = subprocess.check_output(['git', 'show', 'fcc4f254:Cargo.toml'])
env = {k:v for k,v in os.environ.items() if not k.startswith('CARGO_PROFILE_') and k not in ['RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC_BOOTSTRAP', 'NAPI_RS_NATIVE_LIBRARY_PATH']}
env.update(RUSTUP_TOOLCHAIN='1.99.0', CARGO_ENCODED_RUSTFLAGS='', CARGO_TARGET_DIR=os.environ.get('FEATURE_TARGET_DIR', str(out/'target')))
metadata = {'revision': subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(), 'baseline_revision':'fcc4f254', 'target':target, 'rustc':subprocess.check_output(['rustc','+1.99.0','-Vv'],text=True), 'node':subprocess.check_output(['node','--version'],text=True), 'platform':platform.platform(), 'lock_sha256':hashlib.sha256((root/'Cargo.lock').read_bytes()).hexdigest(), 'results':[]}
backups = {}
for c in ['core','parser','codegen']:
    p=root/f'npm/{c}/src-js/swc-next-{c}.{suffix}.node'
    backups[p]=p.read_bytes() if p.exists() else None

def text_section(binary):
    data=binary.read_bytes()
    if is_linux:
        dest=binary.with_suffix('.text.bin')
        subprocess.run(['objcopy','--dump-section',f'.text={dest}',str(binary)],check=True)
        code=dest.read_bytes()
    else:
        assert struct.unpack_from('<I',data)[0]==0xfeedfacf
        offset=32
        code=None
        for _ in range(struct.unpack_from('<I',data,16)[0]):
            cmd,size=struct.unpack_from('<II',data,offset)
            if cmd==0x19:
                for i in range(struct.unpack_from('<I',data,offset+64)[0]):
                    section=offset+72+i*80
                    name=data[section:section+16].split(b'\0')[0]
                    if name==b'__text':
                        length=struct.unpack_from('<Q',data,section+40)[0]
                        file_offset=struct.unpack_from('<I',data,section+48)[0]
                        code=data[file_offset:file_offset+length]
            offset+=size
        assert code is not None
    return {'bytes':len(code),'sha256':hashlib.sha256(code).hexdigest()}

try:
    for side,content in [('base',baseline),('candidate',candidate)]:
        manifest.write_bytes(content)
        dest=out/side;dest.mkdir(exist_ok=True)
        row={'side':side,'manifest_sha256':hashlib.sha256(content).hexdigest(),'artifacts':{}}
        for c in ['core','parser','codegen']:
            tree=subprocess.check_output(['cargo','tree','--locked','--target',target,'-p',f'swc_next_{c}_bridge','-e','normal,build','--prefix','none'],env=env,text=True)
            (dest/f'{c}-tree.txt').write_text(tree)
            features=subprocess.check_output(['cargo','tree','--locked','--target',target,'-p',f'swc_next_{c}_bridge','-e','features','-i','hashbrown@0.17.1'],env=env,text=True)
            (dest/f'{c}-features.txt').write_text(features)
            cmd=['cargo','build','--locked','--release','--target',target,'-p',f'swc_next_{c}_bridge']
            print(side,c,'building',flush=True)
            with (dest/f'{c}-build.log').open('w') as log:
                subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
            binary=dest/f'{c}.node'
            shutil.copy2(pathlib.Path(env['CARGO_TARGET_DIR'])/target/f'release/libswc_next_{c}_bridge.{ext}',binary)
            shutil.copy2(binary,root/f'npm/{c}/src-js/swc-next-{c}.{suffix}.node')
            data=binary.read_bytes()
            row['artifacts'][c]={'bytes':len(data),'gzip_bytes':len(gzip.compress(data,compresslevel=9,mtime=0)),'sha256':hashlib.sha256(data).hexdigest(),'text':text_section(binary),'foldhash_in_graph':'foldhash v' in tree,'command':cmd}
            with (dest/f'{c}-sections.txt').open('w') as log:
                subprocess.run(['readelf','-SW',str(binary)] if is_linux else ['otool','-l',str(binary)],stdout=log,check=True)
            print(side,c,row['artifacts'][c],flush=True)
        with (dest/'tests.log').open('w') as log:
            tested=subprocess.run(['pnpm','test'],env=env,stdout=log,stderr=subprocess.STDOUT)
        if tested.returncode: print((dest/'tests.log').read_text(),flush=True)
        tested.check_returncode()
        row['outputs']={}
        for p in sorted((root/'tasks/benchmark/files').iterdir()):
            if p.suffix not in ['.js','.jsx','.ts','.tsx']:continue
            result=subprocess.check_output(['node','--expose-gc','.github/scripts/native-size-linux/profile.mjs','parse',p.name],env={**env,'PROFILE_SMOKE':'1','NAPI_RS_NATIVE_LIBRARY_PATH':str(dest/'core.node')},text=True)
            row['outputs'][p.name]=json.loads(result)['output']
        metadata['results'].append(row)
        (out/'results.json').write_text(json.dumps(metadata,indent=2))
    assert metadata['results'][0]['outputs']==metadata['results'][1]['outputs']
    assert hashlib.sha256((root/'Cargo.lock').read_bytes()).hexdigest()==metadata['lock_sha256']
    metadata['outputs_match']=True
    # Check affected Rust APIs with the candidate, including allocator doctests.
    with (out/'rust-tests.log').open('w') as log:
        tested=subprocess.run(['cargo','test','--locked','-p','swc_next_allocator','-p','swc_next_ecma_semantic'],env=env,stdout=log,stderr=subprocess.STDOUT)
    if tested.returncode: print((out/'rust-tests.log').read_text()[-12000:],flush=True)
    tested.check_returncode()
    metadata['rust_tests_passed']=True
    (out/'results.json').write_text(json.dumps(metadata,indent=2))
finally:
    manifest.write_bytes(candidate)
    for path,data in backups.items():
        if data is None:path.unlink(missing_ok=True)
        else:path.write_bytes(data)
