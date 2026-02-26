import { useQuery } from '@tanstack/react-query';
import { getGridCells } from '../services/api';
import { GridCell } from '../types';
import { DisplayPixel } from '../App'; // Import the shared type
import { useMemo } from 'react';

interface GridProps {
  onPixelSelect: (pixel: DisplayPixel) => void;
}

const GRID_SIZE = 1000;
const PIXEL_SIZE = 16; // 1rem or 16px
const RENDER_SIZE = 32; // For demonstration purposes

export function Grid({ onPixelSelect }: GridProps) {
  const { data } = useQuery({
    queryKey: ['gridCells'],
    queryFn: () => getGridCells(1),
    refetchInterval: 10000,
  });

  const gridMap = useMemo(() => {
    const map = new Map<string, GridCell>();
    data?.forEach(cell => {
      map.set(`${cell.x},${cell.y}`, cell);
    });
    return map;
  }, [data]);

  const handleCellClick = (x: number, y: number) => {
    const cellData = gridMap.get(`${x},${y}`);
    onPixelSelect({
      x,
      y,
      color: cellData?.color || '#374151', // gray-700
      owner: cellData?.owner,
      link: cellData?.link,
    });
  };

  return (
    <div
      className="relative bg-gray-800 grid gap-px"
      style={{
        width: `${GRID_SIZE * PIXEL_SIZE}px`,
        height: `${GRID_SIZE * PIXEL_SIZE}px`,
        gridTemplateColumns: `repeat(${GRID_SIZE}, minmax(0, 1fr))`,
        // A subtle pattern to indicate the grid exists even when empty
        backgroundImage: 'linear-gradient(rgba(255,255,255,0.05) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.05) 1px, transparent 1px)',
        backgroundSize: `${PIXEL_SIZE}px ${PIXEL_SIZE}px`,
      }}
    >
      {/* We only render the colored cells for performance */}
      {data?.map(cell => (
        <div
          key={`${cell.x}-${cell.y}`}
          className="absolute transition-transform hover:scale-125 hover:z-10 cursor-pointer"
          style={{
            left: `${cell.x * PIXEL_SIZE}px`,
            top: `${cell.y * PIXEL_SIZE}px`,
            width: `${PIXEL_SIZE}px`,
            height: `${PIXEL_SIZE}px`,
            backgroundColor: cell.color,
          }}
          onClick={(e) => {
            e.stopPropagation(); // Prevent click from bubbling to the container
            handleCellClick(cell.x, cell.y);
          }}
        />
      ))}
    </div>
  );
}
