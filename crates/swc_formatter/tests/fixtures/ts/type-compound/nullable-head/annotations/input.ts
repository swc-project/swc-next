type Reference = /*#__PURE__*/ A | null;
type Cast = /** @type {A} */ (A) | null;
type Object = /*@__PURE__*/ { a: 1 } | void;
type Reversed = /*#__NO_SIDE_EFFECTS__*/
  null | A;
