const enum E { A = 'a' }
const enum E { B = 'b' }
const enum E { C = E.A + E.B }
console.log(E.C);