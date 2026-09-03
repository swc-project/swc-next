import { defineConfig } from "@rstest/core";

const browser = {
  enabled: true,
  headless: true,
  provider: "playwright",
};

export default defineConfig({
  projects: [
    browserProject("default", ["import", "default"]),
    browserProject("browser", ["browser", "import", "default"]),
  ],
});

function browserProject(name, conditionNames) {
  return {
    name,
    include: ["npm/core-wasm/test/browser.test.js"],
    browser,
    resolve: { conditionNames },
  };
}
