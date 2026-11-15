async function f() { await /* @__NO_SIDE_EFFECTS__ */ using resource = () => dispose(); }
