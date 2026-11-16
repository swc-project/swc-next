let a=0,b=1; const x1=<Box prop={a=b+1}>{a?b:2}{(a,b)}{...a?b:[]}</Box>;
const a1=(<A/>).x; const b1=(<A/>)(x); const c1=new (<A/>)(); const d=!(<A/>); const e=(<A/>)<b;
const a2=<Box slot=<Inner value={x}/> other=<><One/><Two/></>/>;
const jsxTag=(<Tag enabled/>)`head ${value+1} tail`;
const fragmentTag=(<>head {value}</>)`tail`;
const joined=<Box title={`before ${value+1} after`}>{tag`head ${<Inner/>} tail`}</Box>;
