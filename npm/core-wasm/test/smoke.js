export function smokeTest(core, equal) {
  const { program } = core.parseSync("const value = 1;");
  program.body[0].declarations[0].init.value = 2;

  equal(core.printSync(program).code, "const value = 2;\n");
}
