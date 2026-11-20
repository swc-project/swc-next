type T = string;
function f(
  arg = function (...args: T[]) {
    type T = number;
  },
) {}
