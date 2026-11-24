import assert from "node:assert/strict";
import test from "node:test";
import { builders } from "prettier/doc";
import { encodeDoc, readDocument } from "@swc-next/protocol/document";
import { decodeDoc } from "../src-js/decode.js";

function appendLabelData(buffer, ...records) {
  const length = records.reduce((size, record) => size + 4 + record.length, 4);
  const framed = new Uint8Array(buffer.length + 4 + length + 1).subarray(1);
  const view = new DataView(
    framed.buffer,
    framed.byteOffset,
    framed.byteLength,
  );
  framed.set(buffer);
  let offset = buffer.length;
  view.setUint32(offset, length, true);
  view.setUint32(offset + 4, records.length, true);
  offset += 8;
  for (const record of records) {
    view.setUint32(offset, record.length, true);
    offset += 4;
    framed.set(record, offset);
    offset += record.length;
  }
  return framed;
}

test("reads SWCD label records and retains external label identities", () => {
  const opaque = {};
  const symbol = Symbol();
  const shared = builders.label(opaque, "doc");
  const { buffer, labels } = encodeDoc([
    shared,
    shared,
    builders.label(symbol, "other"),
  ]);
  assert.deepEqual(readDocument(buffer).labelData, []);
  const decoded = decodeDoc(buffer, undefined, labels);
  assert.equal(decoded[0].label, opaque);
  assert.equal(decoded[0], decoded[1]);
  assert.equal(decoded[2].label, symbol);

  const record = Uint8Array.of(255, 0);
  const empty = new Uint8Array();
  const framed = appendLabelData(buffer, record, empty);
  const { labelData } = readDocument(framed);
  assert.deepEqual(labelData, [record, empty]);
  assert.equal(labelData[0].buffer, framed.buffer);
  assert.equal(labelData[0].byteOffset, framed.byteOffset + buffer.length + 12);
  assert.deepEqual(
    decodeDoc(framed).map(({ label }) => label),
    [0, 0, 1],
  );
});

test("rejects malformed SWCD label tables and invalid label references", () => {
  const label = builders.label({}, "doc");
  const { buffer } = encodeDoc(label);
  const framed = appendLabelData(buffer, Uint8Array.of(7));
  for (let end = buffer.length + 1; end < framed.length; end++) {
    assert.throws(() => readDocument(framed.subarray(0, end)));
  }
  assert.throws(() => readDocument(appendLabelData(buffer)));

  const badLength = framed.slice();
  new DataView(badLength.buffer).setUint32(buffer.length + 8, 2, true);
  assert.throws(() => readDocument(badLength));

  const trailing = new Uint8Array(framed.length + 1);
  trailing.set(framed);
  new DataView(trailing.buffer).setUint32(
    buffer.length,
    trailing.length - buffer.length - 4,
    true,
  );
  assert.throws(() => decodeDoc(trailing));

  const twoLabels = encodeDoc([label, builders.label({}, "other")]).buffer;
  assert.throws(() => decodeDoc(appendLabelData(twoLabels, new Uint8Array())));
});
