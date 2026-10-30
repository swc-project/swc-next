import assert from 'node:assert/strict';
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { performance } from 'node:perf_hooks';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { fixtures, prepare } from '../../../tasks/benchmark/js/runner.mjs';
const root = process.cwd();
const sourceDir = resolve(root, 'tasks/benchmark/files');
const script = resolve(root, '.github/scripts/native-size-linux/bench.mjs');
const median = values => [...values].sort((a,b)=>a-b)[Math.floor(values.length/2)];
if (process.argv[2] === '--worker') {
  const { operation, file, timeMs } = JSON.parse(process.argv[3]);
  const api = await import(pathToFileURL(resolve(root,'npm/core/src-js/index.js')));
  const source = readFileSync(resolve(sourceDir,file),'utf8');
  let workload;
  if (operation === 'lazy') {
    const options = { comments: 'none', lang: api.langFromPath(file), preserveParens: true, sourceType: api.sourceTypeFromPath(file) };
    workload = { run: () => api.parseSync(source,options).program.body.length, validate: value => assert(value > 0) };
  } else workload = prepare(api,operation,file,source);
  const initial = workload.run(); workload.validate(initial);
  const fingerprint = createHash('sha256').update(JSON.stringify(initial)).digest('hex');
  let warmupMs=0;
  for(let i=0;i<5;i++){const start=performance.now();workload.run();warmupMs+=performance.now()-start;}
  const cycles=Math.max(1,Math.round(10/Math.max(warmupMs/5,0.001)));
  const samples=[]; let last;
  const deadline=performance.now()+timeMs;
  while(samples.length<5 || performance.now()<deadline){
    const start=performance.now();
    for(let i=0;i<cycles;i++) last=workload.run();
    samples.push((performance.now()-start)/cycles);
  }
  workload.validate(last);
  process.stdout.write(JSON.stringify({bestMs:Math.min(...samples),medianMs:median(samples),samples,cycles,fingerprint}));
} else {
  const [base,candidate,out,kind='comparison'] = process.argv.slice(2);
  const cases = kind === 'comparison'
    ? [...readdirSync(sourceDir).filter(n=>/\.(js|jsx|ts|tsx)$/.test(n)).sort().map(file=>({operation:'lazy',file})), ...fixtures.flatMap(file=>['parse','print'].map(operation=>({operation,file})))]
    : fixtures.map(file=>({operation:'parse',file}));
  const trials=kind==='comparison'?7:3;
  const timeMs=kind==='comparison'?750:500;
  const result={node:process.version,kind,trials,timeMs,method:'Median of best ~10ms rounds; 5 warmup rounds; fresh process per sample; alternating AB/BA order',cases:[]};
  for(const item of cases){
    const row={...item,base:[],candidate:[]};
    for(let trial=0;trial<trials;trial++){
      for(const side of trial%2?['candidate','base']:['base','candidate']){
        const child=spawnSync(process.execPath,[script,'--worker',JSON.stringify({...item,timeMs})],{encoding:'utf8',maxBuffer:8*1024*1024,env:{...process.env,NAPI_RS_NATIVE_LIBRARY_PATH:side==='base'?base:candidate}});
        if(child.error || child.status!==0) throw new Error(`${item.operation}/${item.file} ${side}: ${child.error??child.stderr}`);
        row[side].push(JSON.parse(child.stdout));
      }
    }
    assert.equal(new Set([...row.base,...row.candidate].map(x=>x.fingerprint)).size,1,'Output mismatch');
    row.baseMs=median(row.base.map(x=>x.bestMs));row.candidateMs=median(row.candidate.map(x=>x.bestMs));
    row.deltaPct=(row.candidateMs/row.baseMs-1)*100;
    row.baseRange=[Math.min(...row.base.map(x=>x.bestMs)),Math.max(...row.base.map(x=>x.bestMs))];
    row.candidateRange=[Math.min(...row.candidate.map(x=>x.bestMs)),Math.max(...row.candidate.map(x=>x.bestMs))];
    result.cases.push(row);writeFileSync(out,JSON.stringify(result,null,2));
    console.log(`${kind} ${item.operation}/${item.file}: ${row.deltaPct.toFixed(2)}% (${row.baseMs.toFixed(4)} -> ${row.candidateMs.toFixed(4)} ms)`);
  }
}
