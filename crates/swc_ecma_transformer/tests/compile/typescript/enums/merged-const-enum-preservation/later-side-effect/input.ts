const enum E { A = 1 }
const enum E { B = bump() }
console.log(E.A);