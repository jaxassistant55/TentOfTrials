 ```diff
--- a/frontend/src/components/OrderBook.tsx
+++ b/frontend/src/components/OrderBook.tsx
@@ -1,4 +1,4 @@
-import React, { useMemo, useState, useCallback } from 'react';
+import React, { useMemo, useState, useCallback, useRef, useEffect } from 'react';
 
 // Types
 export interface OrderLevel {
@@ -15,6 +15,10 @@ interface OrderBookProps {
   className?: string;
 }
 
+// Constants for virtualization
+const ROW_HEIGHT = 32; // pixels per row
+const VISIBLE_ROW_COUNT = 25; // number of visible rows to render
+const OVERSCAN_COUNT = 5; // extra rows to render above/below for smooth scrolling
+
 /**
  * OrderBook Component
  * 
@@ -22,7 +26,8 @@ interface OrderBookProps {
  * - Displays bid and ask levels with price, size, and total
  * - Color-codes rows by size relative to max visible size
  * - Supports keyboard navigation (arrow keys, home/end)
- * - TODO: Virtualize rendering for large order books
+ * - Virtualizes rendering for large order books to maintain performance
+ * - Preserves accessibility semantics and visible totals
  */
 export const OrderBook: React.FC<OrderBookProps> = ({
   bids,
@@ -33,6 +38,10 @@ export const OrderBook: React.FC<OrderBookProps> = ({
 }) => {
   const [selectedIndex, setSelectedIndex] = useState<number>(-1);
   const [focusedSide, setFocusedSide] = useState<'bids' | 'asks'>('bids');
+  const [scrollTop, setScrollTop] = useState(0);
+  const bidsContainerRef = useRef<HTMLDivElement>(null);
+  const asksContainerRef = useRef<HTMLDivElement>(null);
+  const containerRefs = useRef<{ bids: HTMLDivElement | null; asks: HTMLDivElement | null }>({ bids: null, asks: null });
 
   // Sort bids descending (highest price first), asks ascending (lowest price first)
   const sortedBids = useMemo(() => {
@@ -48,6 +57,73 @@ export const OrderBook: React.FC<OrderBookProps> = ({
     return sortedAsks.map((ask, index) => ({ ...ask, total: sortedAsks.slice(0, index + 1).reduce((sum, a) => sum + a.size, 0) }));
   }, [sortedAsks]);
 
+  // Virtualization helpers
+  const getVirtualRange = useCallback((itemCount: number, scrollPosition: number) => {
+    const startIndex = Math.max(0, Math.floor(scrollPosition / ROW_HEIGHT) - OVERSCAN_COUNT);
+    const visibleEndIndex = Math.min(
+      itemCount,
+      Math.ceil((scrollPosition + VISIBLE_ROW_COUNT * ROW_HEIGHT) / ROW_HEIGHT) + OVERSCAN_COUNT
+    );
+    const endIndex = Math.min(itemCount, visibleEndIndex);
+    return { start signalIndex: startIndex, endIndex };
+  }, []);
+
+  const getVirtualStyles = useCallback((itemCount: number清算, startIndex: number, visibleCount: number) => {
+    const totalHeight = itemCount * ROW_HEIGHT;
+    const topHeight = startIndex * ROW_HEIGHT;
+    const bottomHeight = Math.max(0, totalHeight - top VirtualHeight - topHeight);
+    
+    return {
+      topHeight,
+      bottomHeight,
+      totalHeight,
+    };
+  }, []);
+
+  // Handle scroll events
+  const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
+    const target = e.currentTarget;
+    const isBids = target === containerRefs.current.bids;
+    if (isBids) {
+      setScrollTop(target.scrollTop);
+    } else {
+      // For asks, we use a separate scroll state or shared - using shared for simplicity
+      setScrollTop(target.scrollTop);
+    }
+  }, []);
+
+  // Virtualized rendering for a side of the book
+  const renderVirtualList = useCallback((
+    items: (OrderLevel & { total: number })[],
+    side: 'bids' | 'asks',
+    maxSize: number
+  ) => {
+    const { startIndex, endIndex } = getVirtualRange(items.length, scrollTop);
+    const visibleItems = items.slice(startIndex, endIndex);
+    const { topHeight, bottomHeight } = getVirtualStyles(items.length, startIndex, visibleItems.length);
+
+    return (
+      <div
+        style={{
+          height: VISIBLE_ROW_COUNT * ROW_HEIGHT,
+          overflow: 'auto',
+          position: 'relative',
+        }}
+        onScroll={handleScroll}
+        ref={side === 'bids' ? (el) => { containerRefs.current.bids = el; bidsContainerRef.current = el; } : (el) => { containerRefs.current.asks = el; asksContainerRef.current = el; }}
+      >
+        <div style={{ height: topHeight }} />
+        {visibleItems.map((item, idx) => {
+          const actualIndex = startIndex + idx;
+          return renderRow(item, side, actualIndex, maxSize, items.length);
+        })}
+        <div style={{ height: bottomHeight }} />
+      </div>
+    );
+  }, [scrollTop, getVirtualRange, getVirtualStyles, handleScroll]);
+
   // Calculate max size for color scaling
   const maxBidSize = useMemo(() => Math.max(...bidsWithTotals.map(b => b.size), 1), [bidsWithTotals]);
   const maxAskSize = useMemo(() => Math.max(...asksWithTotals.map(a => a.size), 1), [asksWithTotals]);
@@ -56,7 +132,7 @@ export const OrderBook: React. = ({
   const getSizeColor = useCallback((size: number, maxSize: number, side: 'bid' | 'ask') => {
     const intensity = Math.min(size / maxSize, 1);
     if (side === 'bid') {
-      return `rgba(0, 128, 0, ${0.1 + intensity * 0.4})`;
+      return `rgba(0, 128, 0, ${0.1 + intensity * 0.4})`; 
     }
     return `rgba(255, 0, 0, ${0.1 + intensity * 0.4})`;
   }, []);
@@ -66,7 +142,7 @@ export const OrderBook: React.FC<OrderBookProps> = ({
     if (e.key === 'ArrowDown') {
       e.preventDefault();
       setSelectedIndex