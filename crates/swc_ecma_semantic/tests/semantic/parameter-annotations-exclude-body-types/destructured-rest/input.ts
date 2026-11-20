type T = string;
function f(...[arg]: T[]) {
  type T = number;
}
