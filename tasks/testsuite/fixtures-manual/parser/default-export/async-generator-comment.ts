/*---
description: Default-exported async generators retain module bindings after a multiline comment
flags: [module]
---*/
export default /* marker
 continued */ async function* f() {}
export { f };
