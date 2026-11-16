const choose = () => { console.log("unexpected"); return "unused"; }; const task = import((/* #__PURE__ */ choose(), "m"));
