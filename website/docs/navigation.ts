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
        "text": "Compatibility",
        "link": "/guide/compatibility"
      }
    ]
  },
  {
    "text": "Getting started",
    "items": [
      {
        "text": "Install",
        "link": "/guide/install"
      },
      {
        "text": "Parser",
        "link": "/guide/parser"
      },
      {
        "text": "Walk",
        "link": "/guide/walk"
      },
      {
        "text": "Codegen",
        "link": "/guide/codegen"
      }
    ]
  },
  {
    "text": "Benchmark",
    "link": "/guide/benchmark"
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
          "text": "Parser",
          "link": "/api/javascript/parser"
        },
        {
          "text": "Walk",
          "link": "/api/javascript/walk"
        },
        {
          "text": "Codegen",
          "link": "/api/javascript/codegen"
        },
        {
          "text": "AST types",
          "link": "/api/javascript/types"
        }
      ]
    },
    {
      "text": "Compiler components",
      "items": [
        {
          "text": "Overview",
          "link": "/api/components/"
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
      "text": "Sponsors",
      "link": "/resources/sponsors"
    },
    {
      "text": "Customers",
      "link": "/resources/customers"
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
