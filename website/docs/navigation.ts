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
  },
  {
    "text": "Performance",
    "link": "/guide/publish-results"
  },
  {
    "text": "Testing",
    "link": "/guide/testing"
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
      "text": "Interop",
      "items": [
        {
          "text": "AST interop",
          "link": "/api/interop"
        }
      ]
    }
  ],
  "/architecture/": [
    {
      "text": "Architecture",
      "items": [
        {
          "text": "Flat AST",
          "link": "/architecture/flat-ast"
        },
        {
          "text": "FFI",
          "link": "/architecture/ffi"
        }
      ]
    }
  ],
  "/resources/": [
    {
      "text": "Team",
      "link": "/resources/team"
    },
    {
      "text": "Roadmap",
      "link": "https://github.com/swc-project/swc-next/issues/181"
    },
    {
      "text": "Releases",
      "link": "https://github.com/swc-project/swc-next/releases"
    }
  ]
};
