const a1=<ns.Box<string> value="ok"/>; const b1=<ns.Box<number> value={1}>value</ns.Box>;
const a2=<ns.Box /* name */ < /* arg */ string /* close */ > value={x}/>;
const joinedTyped=<ns.Box<number> value={tag<number>`raw ${value+1}`}/>;
