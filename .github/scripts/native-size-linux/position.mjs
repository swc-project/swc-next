import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { createRequire } from 'node:module';
import { performance } from 'node:perf_hooks';
import { spawnSync } from 'node:child_process';
import { encodeOptions } from '../../../packages/decoder/index.js';
const script = resolve('.github/scripts/native-size-linux/position.mjs');
const median = xs => {const x=[...xs].sort((a,b)=>a-b);return (x[Math.floor((x.length-1)/2)]+x[Math.floor(x.length/2)])/2;};
if (process.argv[2] === '--worker') {
  const {file,operation,timeMs,warmupMs} = JSON.parse(process.argv[3]);
  const api = await import('../../../npm/core/src-js/index.js');
  const source = readFileSync(resolve('tasks/benchmark/files',file),'utf8');
  const options = {comments:'none',lang:api.langFromPath(file),sourceType:api.sourceTypeFromPath(file),preserveParens:true};
  let run;
  if(operation==='native') {
    const binding=createRequire(import.meta.url)(process.env.NAPI_RS_NATIVE_LIBRARY_PATH);
    const bytes=new TextEncoder().encode(source), encoded=encodeOptions(options);
    run=()=>binding.parseSync(bytes,source.length,encoded).length;
  } else run=()=>api.parseSync(source,options).program.body.length;
  assert(run()>0);
  const startWarm=performance.now();let count=0;
  do {run();count++;} while(performance.now()-startWarm<warmupMs || count<5);
  const cycles=Math.max(1,Math.round(10/((performance.now()-startWarm)/count)));
  const samples=[];const deadline=performance.now()+timeMs;
  do {globalThis.gc();const start=performance.now();for(let i=0;i<cycles;i++)run();samples.push((performance.now()-start)/cycles);} while(performance.now()<deadline || samples.length<10);
  process.stdout.write(JSON.stringify({bestMs:Math.min(...samples),medianMs:median(samples),cycles,samples}));
} else {
  const [base,candidate,out]=process.argv.slice(2);
  const result={node:process.version,method:'Fresh processes, alternating AB/BA, >=1s warmup, GC outside timed rounds, median of 7 per-process best ~10ms rounds',cases:[]};
  const files=['astro-config.ts','cal.com.tsx','backbone-1.1.0.js','jquery-1.9.1.js','position-sparse.js','position-dense.js'];
  for(const kind of ['control-before','comparison','control-after']) {
    const cases=kind==='comparison'?files.flatMap(file=>['native','lazy'].map(operation=>({file,operation}))):['astro-config.ts','cal.com.tsx'].map(file=>({file,operation:'native'}));
    for(const item of cases) {
      const trials=kind==='comparison'?7:3;
      const row={kind,...item,base:[],candidate:[]};
      for(let trial=0;trial<trials;trial++) for(const side of trial%2?['candidate','base']:['base','candidate']) {
        const binary=kind==='control-before'?base:kind==='control-after'?candidate:side==='base'?base:candidate;
        const child=spawnSync(process.execPath,['--expose-gc',script,'--worker',JSON.stringify({...item,timeMs:750,warmupMs:1000})],{encoding:'utf8',env:{...process.env,NAPI_RS_NATIVE_LIBRARY_PATH:binary}});
        if(child.status!==0)throw new Error(child.stderr);
        row[side].push(JSON.parse(child.stdout));
      }
      row.baseMs=median(row.base.map(x=>x.bestMs));row.candidateMs=median(row.candidate.map(x=>x.bestMs));row.deltaPct=(row.candidateMs/row.baseMs-1)*100;
      for(const side of ['base','candidate'])row[`${side}Range`]=[Math.min(...row[side].map(x=>x.bestMs)),Math.max(...row[side].map(x=>x.bestMs))];
      result.cases.push(row);writeFileSync(out,JSON.stringify(result,null,2));console.log(kind,item.operation,item.file,row.deltaPct.toFixed(3));
    }
  }
}
