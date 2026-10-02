result = Factory.first
// prettier-ignore
 ({ one :1, two:2 }, [(arg) => { trace.push(arg); return arg; }]).second(other);
