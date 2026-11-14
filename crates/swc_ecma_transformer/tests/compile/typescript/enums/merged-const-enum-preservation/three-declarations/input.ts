const enum E { A = 1 }
const enum E { B = bump() }
const enum E { C = 3 }
console.log(E.A, E.C);