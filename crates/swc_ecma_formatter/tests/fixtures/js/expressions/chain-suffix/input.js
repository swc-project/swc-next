const entry = collection.read(first).select(second)[0].record;
consume(collection.read(first).select(second)[0].record.value);
collection.read(first).select(second)[recordKey].record.value;
consume(collection.read(first).select(second)
  // receiver note
  .record.value);
consume(collection.read(first).select(second)

  .record.value);
const annotated = (/*#__PURE__*/ makeCollection(first).select(second))[0].record;
const ignored = collection.read( first ).select( second ) /* prettier-ignore */ .record;
collection.read(first).select(second)[0].record;
[entry].map(consume);
