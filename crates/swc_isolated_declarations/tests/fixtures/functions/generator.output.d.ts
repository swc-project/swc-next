// Correct
declare function generatorGood(): Generator<number>;
declare class GeneratorClassGood {
  method(): Generator<number>;
}
// Need to explicit return type for async functions
// Incorrect
declare function generatorBad();
declare class GeneratorClassBad {
  method();
}
