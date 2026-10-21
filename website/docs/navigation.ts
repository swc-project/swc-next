const guide = [
  {
    "text": "Introduction",
    "items": [
      {
        "text": "Overview",
        "link": "/"
      },
      {
        "text": "Project status",
        "link": "/guide/status"
      },
      {
        "text": "Compatibility with SWC",
        "link": "/guide/compatibility"
      }
    ]
  },
  {
    "text": "Getting started",
    "items": [
      {
        "text": "Node.js",
        "link": "/guide/getting-started/node"
      },
      {
        "text": "Browser & WebAssembly",
        "link": "/guide/getting-started/browser"
      },
      {
        "text": "Rust",
        "link": "/guide/getting-started/rust"
      },
      {
        "text": "Choosing packages",
        "link": "/guide/packages"
      }
    ]
  },
  {
    "text": "Working with ASTs",
    "items": [
      {
        "text": "Parse, walk, and print",
        "link": "/guide/working-with-asts"
      },
      {
        "text": "Diagnostics & positions",
        "link": "/guide/diagnostics"
      },
      {
        "text": "TypeScript & JSX",
        "link": "/guide/typescript-jsx"
      },
      {
        "text": "Common recipes",
        "link": "/guide/recipes"
      }
    ]
  }
];

export const sidebar = {
  "/": guide,
  "/guide/": guide,
  "/api/": [
    {
      "text": "API reference",
      "items": [
        {
          "text": "Overview",
          "link": "/api/"
        },
        {
          "text": "Component APIs",
          "link": "/api/components/"
        }
      ]
    },
    {
      "text": "Compiler components",
      "items": [
        {
          "text": "Parser",
          "link": "/api/components/parser"
        },
        {
          "text": "AST & traversal",
          "link": "/api/components/ast"
        },
        {
          "text": "Semantic analysis",
          "link": "/api/components/semantic"
        },
        {
          "text": "Transformer",
          "link": "/api/components/transformer"
        },
        {
          "text": "Code generation",
          "link": "/api/components/codegen"
        },
        {
          "text": "Minifier",
          "link": "/api/components/minifier"
        },
        {
          "text": "Type strip",
          "link": "/api/components/type-strip"
        },
        {
          "text": "Isolated declarations",
          "link": "/api/components/isolated-declarations"
        }
      ]
    },
    {
      "text": "JavaScript",
      "items": [
        {
          "text": "Core",
          "link": "/api/javascript/core"
        },
        {
          "text": "Parser & codegen",
          "link": "/api/javascript/parser-codegen"
        },
        {
          "text": "Browser API",
          "link": "/api/javascript/browser"
        },
        {
          "text": "Walker",
          "link": "/api/javascript/walker"
        },
        {
          "text": "AST types",
          "link": "/api/javascript/types"
        }
      ]
    },
    {
      "text": "Rust & interop",
      "items": [
        {
          "text": "Rust API",
          "link": "/api/rust"
        },
        {
          "text": "AST interop",
          "link": "/api/interop"
        }
      ]
    }
  ],
  "/benchmarks/": [
    {
      "text": "Performance",
      "items": [
        {
          "text": "Overview",
          "link": "/benchmarks/"
        },
        {
          "text": "Published results",
          "link": "/benchmarks/results"
        },
        {
          "text": "Rust benchmarks",
          "link": "/benchmarks/rust"
        },
        {
          "text": "JavaScript API",
          "link": "/benchmarks/javascript"
        },
        {
          "text": "Memory",
          "link": "/benchmarks/memory"
        },
        {
          "text": "Methodology",
          "link": "/benchmarks/methodology"
        }
      ]
    }
  ],
  "/resources/": [
    {
      "text": "Resources",
      "items": [
        {
          "text": "Overview",
          "link": "/resources/"
        }
      ]
    },
    {
      "text": "Architecture",
      "items": [
        {
          "text": "Flat AST",
          "link": "/resources/architecture/flat-ast"
        },
        {
          "text": "AST representations",
          "link": "/resources/architecture/representations"
        },
        {
          "text": "Schema & generation",
          "link": "/resources/architecture/generation"
        },
        {
          "text": "Native & Wasm bridge",
          "link": "/resources/architecture/bridge"
        }
      ]
    },
    {
      "text": "Quality",
      "items": [
        {
          "text": "Testing & conformance",
          "link": "/resources/testing"
        },
        {
          "text": "Known limitations",
          "link": "/resources/limitations"
        }
      ]
    },
    {
      "text": "Project",
      "items": [
        {
          "text": "Roadmap",
          "link": "/resources/roadmap"
        },
        {
          "text": "Releases",
          "link": "/resources/releases"
        },
        {
          "text": "Contributing",
          "link": "/resources/contributing"
        },
        {
          "text": "FAQ",
          "link": "/resources/faq"
        },
        {
          "text": "Credits & license",
          "link": "/resources/credits"
        }
      ]
    }
  ]
};
