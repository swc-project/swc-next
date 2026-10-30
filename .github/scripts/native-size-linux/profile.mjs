import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { resolve } from 'node:path';
import { encodeOptions as parserOptions } from '../../../packages/decoder/index.js';
import { encode, encodeOptions as codegenOptions } from '../../../packages/encoder/index.js';
import * as api from '../../../npm/core/src-js/index.js';
const [operation,file] = process.argv.slice(2);
const require=createRequire(import.meta.url);
const binding=require(process.env.NAPI_RS_NATIVE_LIBRARY_PATH);
const measure=process.env.PROFILE_SMOKE==='1' ? fn=>fn() : require(process.env.MEASURE_ADDON).measure;
const source=readFileSync(resolve('tasks/benchmark/files',file),'utf8');
const options={lang:api.langFromPath(file),sourceType:api.sourceTypeFromPath(file),comments:'none'};
let run;
if(operation==='parse'){
 const bytes=new TextEncoder().encode(source);const encoded=parserOptions(options);
 run=()=>binding.parseSync(bytes,source.length,encoded);
}else{
 const parsed=api.parseSync(source,options);assert.equal(parsed.diagnostics.length,0);
 const args=[...encode(parsed.program,parsed.meta),codegenOptions({})];
 run=()=>binding.printSync(...args);
}
for(let i=0;i<10;i++)run();
globalThis.gc();
const results=new Array(3);
measure(()=>{for(let i=0;i<results.length;i++)results[i]=run();});
const hashes=results.map(value=>createHash('sha256').update(typeof value==='string'?value:Buffer.from(value)).digest('hex'));
assert.equal(new Set(hashes).size,1);
console.log(JSON.stringify({operation,file,iterations:3,output:hashes[0]}));
