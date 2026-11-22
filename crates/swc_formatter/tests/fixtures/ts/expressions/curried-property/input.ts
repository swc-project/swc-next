const config = {
  setup: ({ server }) => () => {
    server.middlewares.use(handler);
  },
};
