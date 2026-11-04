result = collection.read(1).select(2)[0]!.property.final;
result = collection.read(1).select(2)[0]!.read!().select()!.property.final;
result = collection.read(1).select(2)!.property.final;
result = collection.read(1).select(2).property!.final;
result = (nullable?.read()?.select()!)?.property.read(argument()).select();
result = (nullable?.read()?.select())!?.property.read(argument()).select();
result = (nullable?.read()?.select()!).property.read(argument()).select();
result = (nullable?.read()?.select())!.property.read(argument()).select();
result = collection.read(1).select(2).property!;
result = collection.read(1).select(2)[0] /* assertion gap */ ! // asserted receiver
  .property.final;
result = collection.read(1).select(2)[0] /* prettier-ignore */ !. property  .final;
consume(collection.read(1).select(2)[0]! // @ts-expect-error
  .missing.value);
