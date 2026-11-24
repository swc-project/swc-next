type Checked = (Input)extends
  // prettier-ignore
  Box <  Bound  > ? Yes : No;

type Constrained = Input  [  ]/* prettier-ignore */extends(Bound) ? Yes : No;
