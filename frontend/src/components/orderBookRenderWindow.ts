export const DEFAULT_ORDER_BOOK_VISIBLE_ROWS = 15;
export const MAX_ORDER_BOOK_RENDERED_ROWS = 60;

export interface OrderBookRenderWindow<T> {
  rows: T[];
  rowOffset: number;
  totalRows: number;
  requestedRows: number;
  isVirtualized: boolean;
}

function normalizeRequestedRows(requestedRows: number | undefined): number {
  if (requestedRows === undefined || !Number.isFinite(requestedRows)) {
    return DEFAULT_ORDER_BOOK_VISIBLE_ROWS;
  }
  return Math.max(0, Math.floor(requestedRows));
}

export function createOrderBookRenderWindow<T>(
  levels: readonly T[],
  requestedRows?: number
): OrderBookRenderWindow<T> {
  const normalizedRows = normalizeRequestedRows(requestedRows);
  const renderLimit = Math.min(normalizedRows, MAX_ORDER_BOOK_RENDERED_ROWS);
  const rows = renderLimit === 0 ? [] : levels.slice(0, renderLimit);

  return {
    rows,
    rowOffset: 0,
    totalRows: levels.length,
    requestedRows: normalizedRows,
    isVirtualized: levels.length > rows.length,
  };
}
