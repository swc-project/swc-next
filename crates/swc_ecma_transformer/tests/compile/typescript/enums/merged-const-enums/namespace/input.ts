namespace N { const enum E { A = 1 }
const enum E { B = 2 }
export const value = E.A + E.B; } console.log(N.value);