for (const make = () => {
  for (; "x" in owner; step = "x" in owner) { break; }
  for (let seen = ("x" in owner);;) { break; }
  for (const key in "x" in owner ? owner : {}) { consume(key); }
  for (const value of "x" in owner ? values : []) { consume(value); }
  return "x" in owner;
}; false;) {}
"x" in owner;
