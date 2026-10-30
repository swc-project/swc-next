import gzip, hashlib, json, os, pathlib, re, shutil, subprocess

root = pathlib.Path.cwd()
out = pathlib.Path(os.environ['RUNNER_TEMP']) / 'position-map'
out.mkdir(exist_ok=True)
mode = os.environ['POSITION_MODE']
toolchain = '1.99.0' if mode == 'stable' else 'nightly-2026-10-29'
extra = ['-Zbuild-std=panic_abort,std'] if mode == 'std' else []
source = root / 'crates/swc_ecma_ast/src/ffi/ast.rs'
candidate = source.read_bytes()
baseline = subprocess.check_output(['git', 'show', 'a781afa1:crates/swc_ecma_ast/src/ffi/ast.rs'])
fixtures = root / 'tasks/benchmark/files'
# Exercise both sparse and dense Unicode independently of source language.
(fixtures / 'position-sparse.js').write_text('/*' + 'a' * 251 + '😀é*/\n' + '\n'.join(f'const value{i} = {i} + 1;' for i in range(4000)))
(fixtures / 'position-dense.js').write_text('\n'.join(f'const value{i} = "中文😀é";' for i in range(4000)))
node = pathlib.Path(shutil.which('node'))
addon = out / 'measure.node'
subprocess.run(['cc', '-shared', '-fPIC', '-O2', f'-I{node.parent.parent}/include/node', '-o', str(addon), 'tasks/benchmark/js/measure.c'], check=True)
metadata = {'mode': mode, 'toolchain': toolchain, 'extra': extra, 'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(), 'node': subprocess.check_output(['node', '--version'], text=True).strip(), 'cpu': subprocess.check_output(['lscpu'], text=True), 'results': []}
cleanenv = {k: v for k,v in os.environ.items() if not k.startswith('CARGO_PROFILE_') and k not in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_BOOTSTRAP','NAPI_RS_NATIVE_LIBRARY_PATH']}
env = {**cleanenv, 'RUSTUP_TOOLCHAIN': toolchain, 'CARGO_ENCODED_RUSTFLAGS': '', 'CARGO_TARGET_DIR': str(out / 'target')}
try:
    for side, content in [('base', baseline), ('candidate', candidate)]:
        source.write_bytes(content)
        dest = out / side
        dest.mkdir(exist_ok=True)
        cmd = ['cargo', 'build', '--locked', '--release', '--target', 'x86_64-unknown-linux-gnu', '-p', 'swc_next_core_bridge', *extra]
        print(mode, side, 'building', flush=True)
        with (dest / 'build.log').open('w') as log:
            subprocess.run(cmd, env=env, stdout=log, stderr=subprocess.STDOUT, check=True)
        binary = dest / 'core.node'
        shutil.copy2(out / 'target/x86_64-unknown-linux-gnu/release/libswc_next_core_bridge.so', binary)
        data = binary.read_bytes()
        row = {'side': side, 'bytes': len(data), 'gzip_bytes': len(gzip.compress(data, mtime=0)), 'sha256': hashlib.sha256(data).hexdigest(), 'profiles': []}
        runenv = {**env, 'NAPI_RS_NATIVE_LIBRARY_PATH': str(binary), 'MEASURE_ADDON': str(addon)}
        with (dest / 'tests.log').open('w') as log:
            subprocess.run(['pnpm', 'test'], env=runenv, stdout=log, stderr=subprocess.STDOUT, check=True)
        # Compare complete raw transfers, including spans, for every real and synthetic fixture.
        row['outputs'] = {}
        for fixture in sorted(p.name for p in fixtures.iterdir() if p.suffix in ['.js','.jsx','.ts','.tsx']):
            result = subprocess.check_output(['node', '--expose-gc', '.github/scripts/native-size-linux/profile.mjs', 'parse', fixture], env={**runenv, 'PROFILE_SMOKE': '1'}, text=True)
            row['outputs'][fixture] = json.loads(result)['output']
        for fixture in ['astro-config.ts', 'cal.com.tsx', 'backbone-1.1.0.js', 'position-sparse.js', 'position-dense.js']:
            counts = []
            for trial in range(2):
                profile = dest / f'{fixture}-{trial}.callgrind'
                cmd = ['valgrind','--tool=callgrind','--instr-atstart=no','--collect-atstart=no','--cache-sim=no','--branch-sim=no','--smc-check=all-non-file', f'--callgrind-out-file={profile}', str(node),'--jitless','--predictable','--expose-gc','--random-seed=42','.github/scripts/native-size-linux/profile.mjs','parse',fixture]
                with (dest / f'{fixture}-{trial}.log').open('w') as log:
                    result = subprocess.run(cmd, env=runenv, stdout=subprocess.PIPE, stderr=log, text=True, check=True)
                count = int(re.search(r'^totals:\s+(\d+)', profile.read_text(), re.M)[1])
                assert count > 0
                counts.append(count)
                assert json.loads(result.stdout)['output'] == row['outputs'][fixture]
            row['profiles'].append({'fixture': fixture, 'counts': counts})
            print(mode, side, fixture, counts, flush=True)
        metadata['results'].append(row)
        (out / 'results.json').write_text(json.dumps(metadata, indent=2))
finally:
    source.write_bytes(candidate)
assert metadata['results'][0]['outputs'] == metadata['results'][1]['outputs']
metadata['outputs_match'] = True
cpu = min(os.sched_getaffinity(0))
os.sched_setaffinity(0, {cpu})
metadata['benchmark_cpu'] = cpu
(out / 'results.json').write_text(json.dumps(metadata, indent=2))
with (out / 'timing.log').open('w') as log:
    subprocess.run(['node', '.github/scripts/native-size-linux/position.mjs', str(out / 'base/core.node'), str(out / 'candidate/core.node'), str(out / 'timing.json')], env=cleanenv, stdout=log, stderr=subprocess.STDOUT, check=True)
