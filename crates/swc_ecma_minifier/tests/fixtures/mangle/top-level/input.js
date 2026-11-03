const kept = 1;
function api(param) { return param + kept; }
{ let block = kept; use(block); }
export { kept as value };
export default class Widget { method(param) { return param; } }
