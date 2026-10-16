/*---
description: Template literal types support normalized newlines in nested union slots
flags: [module]
---*/

type T = `
${`
${string | number}
` | "fallback"}
`;
