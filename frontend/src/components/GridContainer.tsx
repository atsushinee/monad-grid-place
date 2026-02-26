import React, { useState, useRef, WheelEvent, MouseEvent, useCallback } from 'react';
import { DisplayPixel } from '../App';

interface GridContainerProps {
  children: React.ReactNode;
  onPixelSelect: (pixel: DisplayPixel) => void;
}

const MIN_SCALE = 0.1;
const MAX_SCALE = 20;
const PIXEL_SIZE = 16;

export function GridContainer({ children, onPixelSelect }: GridContainerProps) {
  const [transform, setTransform] = useState({ scale: 1, translateX: 0, translateY: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const lastMousePosition = useRef({ x: 0, y: 0 });
  const containerRef = useRef<HTMLDivElement>(null);

  const handleWheel = (e: WheelEvent<HTMLDivElement>) => {
    e.preventDefault();
    const { deltaY } = e;
    const scaleAmount = -deltaY / 500;
    const newScale = Math.max(MIN_SCALE, Math.min(MAX_SCALE, transform.scale + scaleAmount));

    const rect = containerRef.current?.getBoundingClientRect();
    if (!rect) return;

    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const newTranslateX = transform.translateX - (mouseX - transform.translateX) * (newScale / transform.scale - 1);
    const newTranslateY = transform.translateY - (mouseY - transform.translateY) * (newScale / transform.scale - 1);

    setTransform({ scale: newScale, translateX: newTranslateX, translateY: newTranslateY });
  };

  const handleMouseDown = (e: MouseEvent<HTMLDivElement>) => {
    // Prevent starting a drag on right-click
    if (e.button !== 0) return;
    
    setIsDragging(true);
    lastMousePosition.current = { x: e.clientX, y: e.clientY };
    e.currentTarget.style.cursor = 'grabbing';
  };

  const handleMouseUp = (e: MouseEvent<HTMLDivElement>) => {
    setIsDragging(false);
    e.currentTarget.style.cursor = 'grab';
  };

  const handleMouseMove = (e: MouseEvent<HTMLDivElement>) => {
    if (!isDragging) return;
    const deltaX = e.clientX - lastMousePosition.current.x;
    const deltaY = e.clientY - lastMousePosition.current.y;
    lastMousePosition.current = { x: e.clientX, y: e.clientY };

    setTransform(prev => ({
      ...prev,
      translateX: prev.translateX + deltaX,
      translateY: prev.translateY + deltaY,
    }));
  };

  const handleClick = useCallback((e: MouseEvent<HTMLDivElement>) => {
    // This logic ensures that a click is not registered after a drag
    const dx = e.clientX - lastMousePosition.current.x;
    const dy = e.clientY - lastMousePosition.current.y;
    if (Math.abs(dx) > 2 || Math.abs(dy) > 2) {
      return; // It was a drag, not a click
    }

    const rect = containerRef.current?.getBoundingClientRect();
    if (!rect) return;

    // Convert screen coordinates to grid coordinates
    const clickX = e.clientX - rect.left;
    const clickY = e.clientY - rect.top;

    const gridX = Math.floor((clickX - transform.translateX) / transform.scale / PIXEL_SIZE);
    const gridY = Math.floor((clickY - transform.translateY) / transform.scale / PIXEL_SIZE);

    if (gridX >= 0 && gridX < 1000 && gridY >= 0 && gridY < 1000) {
       onPixelSelect({ x: gridX, y: gridY, color: '#374151' });
    }

  }, [transform, onPixelSelect]);

  return (
    <div
      ref={containerRef}
      className="w-full h-full bg-gray-900 overflow-hidden relative touch-none"
      style={{ cursor: 'grab' }}
      onWheel={handleWheel}
      onMouseDown={handleMouseDown}
      onMouseUp={handleMouseUp}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseUp}
      onClick={handleClick}
    >
      <div
        className="transform-gpu"
        style={{
          transform: `translate(${transform.translateX}px, ${transform.translateY}px) scale(${transform.scale})`,
          transformOrigin: '0 0',
        }}
      >
        {children}
      </div>
    </div>
  );
}
