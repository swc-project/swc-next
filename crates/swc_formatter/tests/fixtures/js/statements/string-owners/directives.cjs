("barrier");
"use strict";

function strict() {
  'custom';
  'use strict';
  return this;
}

function blocked(value = 0) {
  "custom";
  ("use strict");
  return this;
}

function escaped() {
  "\x75se strict";
  return this;
}
