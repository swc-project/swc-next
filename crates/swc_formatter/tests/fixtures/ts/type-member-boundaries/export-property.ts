interface I {
  export;
  y: number;
}

type T = { export; y: number };

interface Optional {
  export?;
  y: number;
}

interface Last {
  export;
}
