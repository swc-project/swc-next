chain.plugin('extra-define').use(rspack.DefinePlugin, [
  {
    ENABLE_TEST: JSON.stringify(true),
  },
]);
