export function bindingFileNames(binaryName) {
  return {
    wasm: `${binaryName}.wasm`,
    bridge: `${binaryName}.bridge.mjs`,
    node: `${binaryName}.node.mjs`,
    browser: `${binaryName}.browser.mjs`,
    module: `${binaryName}.module.mjs`,
    runtime: `${binaryName}.runtime.mjs`,
  };
}
