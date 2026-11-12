const enum E { A = 1 }
const enum E { B = E.A + 1, C }
console.log(E.A, E.B, E.C);