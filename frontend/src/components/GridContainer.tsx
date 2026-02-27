import React, { useState, useRef, MouseEvent, useCallback, useEffect } from 'react';

export type Area = { startX: number; startY: number; endX: number; endY: number; };

interface GridContainerProps {
  children: React.ReactNode;
  onAreaSelect: (area: Area) => void | Promise<void>;
  onPixelSelect: (x: number, y: number, addToSelection: boolean) => void | Promise<void>;
}

const MIN_SCALE = 0.1;
const MAX_SCALE = 20;
const PIXEL_SIZE = 16;

export function GridContainer({ children, onAreaSelect, onPixelSelect }: GridContainerProps) {
  const [transform, setTransform] = useState({ scale: 1, translateX: 0, translateY: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [isSelecting, setIsSelecting] = useState(false);
  const [selectionArea, setSelectionArea] = useState<Area | null>(null);
  const dragStartPos = useRef({ x: 0, y: 0 });
  const containerRef = useRef<HTMLDivElement>(null);

  const screenToGridCoords = useCallback((screenX: number, screenY: number) => {
    const rect = containerRef.current?.getBoundingClientRect();
    if (!rect) return { x: 0, y: 0 };
    const gridX = Math.floor((screenX - rect.left - transform.translateX) / (transform.scale * PIXEL_SIZE));
    const gridY = Math.floor((screenY - rect.top - transform.translateY) / (transform.scale * PIXEL_SIZE));
    return { x: gridX, y: gridY };
  }, [transform]);

  const handleMouseDown = (e: MouseEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;
    dragStartPos.current = { x: e.clientX, y: e.clientY };

    if (e.shiftKey) {
      // Shift + 拖拽：矩形框选
      setIsSelecting(true);
      const startCoords = screenToGridCoords(e.clientX, e.clientY);
      setSelectionArea({ startX: startCoords.x, startY: startCoords.y, endX: startCoords.x, endY: startCoords.y });
    } else {
      setIsDragging(true);
      e.currentTarget.style.cursor = 'grabbing';
    }
  };

  const handleMouseMove = (e: MouseEvent<HTMLDivElement>) => {
    if (isSelecting) {
      const endCoords = screenToGridCoords(e.clientX, e.clientY);
      setSelectionArea(prev => prev ? { ...prev, endX: endCoords.x, endY: endCoords.y } : null);
    } else if (isDragging) {
      const deltaX = e.clientX - dragStartPos.current.x;
      const deltaY = e.clientY - dragStartPos.current.y;
      dragStartPos.current = { x: e.clientX, y: e.clientY };
      setTransform(prev => ({
        ...prev,
        translateX: prev.translateX + deltaX,
        translateY: prev.translateY + deltaY,
      }));
    }
  };

  const handleMouseUp = async (e: MouseEvent<HTMLDivElement>) => {
    e.currentTarget.style.cursor = 'grab';
    const dx = e.clientX - dragStartPos.current.x;
    const dy = e.clientY - dragStartPos.current.y;

    if (isSelecting) {
      if (selectionArea) {
        const { startX, startY, endX, endY } = selectionArea;
        const finalArea = {
          startX: Math.min(startX, endX),
          startY: Math.min(startY, endY),
          endX: Math.max(startX, endX),
          endY: Math.max(startY, endY),
        };
        await onAreaSelect(finalArea);
      }
      setSelectionArea(null);
    } else if (isDragging) {
      // 点击（没有拖动）：选择像素
      if (Math.abs(dx) < 3 && Math.abs(dy) < 3) {
        const coords = screenToGridCoords(e.clientX, e.clientY);
        // Ctrl/Cmd + 点击：多选模式
        const addToSelection = e.ctrlKey || e.metaKey;
        await onPixelSelect(coords.x, coords.y, addToSelection);
      }
    }

    setIsDragging(false);
    setIsSelecting(false);
  };

  // Use useEffect to add a non-passive event listener for the wheel event
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const handleWheel = (e: globalThis.WheelEvent) => {
      e.preventDefault();
      const { deltaY } = e;
      const scaleAmount = -deltaY / 500;

      setTransform(prevTransform => {
        const newScale = Math.max(MIN_SCALE, Math.min(MAX_SCALE, prevTransform.scale + scaleAmount));
        const rect = container.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;
        const newTranslateX = prevTransform.translateX - (mouseX - prevTransform.translateX) * (newScale / prevTransform.scale - 1);
        const newTranslateY = prevTransform.translateY - (mouseY - prevTransform.translateY) * (newScale / prevTransform.scale - 1);
        return { scale: newScale, translateX: newTranslateX, translateY: newTranslateY };
      });
    };

    container.addEventListener('wheel', handleWheel, { passive: false });
    return () => container.removeEventListener('wheel', handleWheel);
  }, []);


  return (
    <div
      ref={containerRef}
      className="w-full h-full bg-[#1A1A1A] overflow-hidden relative touch-none"
      style={{ cursor: 'grab' }}
      onMouseDown={handleMouseDown}
      onMouseUp={handleMouseUp}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseUp}
    >
      <div
        className="transform-gpu"
        style={{
          transform: `translate(${transform.translateX}px, ${transform.translateY}px) scale(${transform.scale})`,
          transformOrigin: '0 0',
        }}
      >
        {children}
        {selectionArea && (
          <div
            className="absolute bg-green-500/20 border-2 border-dashed border-[#00FFAA] pointer-events-none"
            style={{
              left: `${Math.min(selectionArea.startX, selectionArea.endX) * PIXEL_SIZE}px`,
              top: `${Math.min(selectionArea.startY, selectionArea.endY) * PIXEL_SIZE}px`,
              width: `${(Math.abs(selectionArea.startX - selectionArea.endX) + 1) * PIXEL_SIZE}px`,
              height: `${(Math.abs(selectionArea.startY - selectionArea.endY) + 1) * PIXEL_SIZE}px`,
            }}
          />
        )}
      </div>
    </div>
  );
}
