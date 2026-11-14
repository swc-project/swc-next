enum E { A = 1, B = (() => { for (A of [2]) {} })() }
