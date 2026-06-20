import assert from 'node:assert/strict';
import test from 'node:test';

import {
  MAX_ORDER_BOOK_RENDERED_ROWS,
  createOrderBookRenderWindow,
} from '../src/components/orderBookRenderWindow.js';

function levels(count: number) {
  return Array.from({ length: count }, (_, index) => ({
    price: 100 - index,
    size: index + 1,
    total: index + 1,
  }));
}

test('empty books render no visible rows without virtualizing', () => {
  const window = createOrderBookRenderWindow(levels(0), 15);

  assert.equal(window.totalRows, 0);
  assert.equal(window.rows.length, 0);
  assert.equal(window.isVirtualized, false);
});

test('small books render every level', () => {
  const source = levels(8);
  const window = createOrderBookRenderWindow(source, 15);

  assert.equal(window.totalRows, 8);
  assert.equal(window.rows.length, 8);
  assert.equal(window.isVirtualized, false);
  assert.deepEqual(window.rows, source);
});

test('large books render a bounded window instead of one row per level', () => {
  const window = createOrderBookRenderWindow(levels(2_000), 500);

  assert.equal(window.totalRows, 2_000);
  assert.equal(window.rows.length, MAX_ORDER_BOOK_RENDERED_ROWS);
  assert.equal(window.isVirtualized, true);
});

test('visible row positions stay stable when visible levels update in place', () => {
  const initial = levels(200);
  const updated = initial.map((level, index) => (
    index < 20 ? { ...level, size: level.size + 0.5, total: level.total + 0.5 } : level
  ));

  const before = createOrderBookRenderWindow(initial, 20);
  const after = createOrderBookRenderWindow(updated, 20);

  assert.deepEqual(
    before.rows.map(level => level.price),
    after.rows.map(level => level.price)
  );
  assert.equal(after.rows.length, 20);
  assert.equal(after.isVirtualized, true);
});
