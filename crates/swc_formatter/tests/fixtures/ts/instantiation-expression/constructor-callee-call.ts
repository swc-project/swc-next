new (factory()<Item>)();
new (factory()!.namespace.Constructor<Item>)();
new (factory()[key]<Item>)();
new ((factory()?.Constructor)<Item>)();
