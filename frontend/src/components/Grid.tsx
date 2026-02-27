import { useQuery } from '@tanstack/react-query';
import { getGridCells } from '../services/api';
import { DisplayPixel } from '../types';
import { useMemo } from 'react';

interface GridProps {
  selectedPixels: DisplayPixel[];
  onPixelClick?: (x: number, y: number, addToSelection: boolean) => void;
}

const GRID_SIZE = 1000;
const PIXEL_SIZE = 16;

export function Grid({ selectedPixels, onPixelClick }: GridProps) {
  const { data } = useQuery({
    queryKey: ['gridCells'],
    queryFn: () => getGridCells(1, 10000), // 获取最多 10000 个像素
    refetchInterval: 5000, // 5 秒刷新一次
    refetchOnWindowFocus: true, // 窗口聚焦时刷新
  });

  // 创建已选像素的 Map 用于快速查找
  const selectedMap = useMemo(() => {
    const map = new Map<string, boolean>();
    selectedPixels.forEach(p => map.set(`${p.x},${p.y}`, true));
    return map;
  }, [selectedPixels]);

  // 创建已涂色像素的 Map（有颜色的像素）
  const coloredMap = useMemo(() => {
    const map = new Map<string, string>();
    data?.forEach(cell => {
      if (cell.color && cell.color !== '#1A1A1A') {
        map.set(`${cell.x},${cell.y}`, cell.color);
      }
    });
    return map;
  }, [data]);

  // 获取需要显示边框的边缘
  const edges = useMemo(() => {
    const edgeSet = new Set<string>();
    
    selectedPixels.forEach(pixel => {
      const x = pixel.x;
      const y = pixel.y;

      // 检查四个方向
      const hasTop = selectedMap.has(`${x},${y - 1}`);
      const hasBottom = selectedMap.has(`${x},${y + 1}`);
      const hasLeft = selectedMap.has(`${x - 1},${y}`);
      const hasRight = selectedMap.has(`${x + 1},${y}`);

      // 添加边缘（格式：x,y,方向）
      if (!hasTop) edgeSet.add(`${x},${y},top`);
      if (!hasBottom) edgeSet.add(`${x},${y},bottom`);
      if (!hasLeft) edgeSet.add(`${x},${y},left`);
      if (!hasRight) edgeSet.add(`${x},${y},right`);
    });

    return edgeSet;
  }, [selectedPixels, selectedMap]);

  // 生成 SVG 路径
  const svgPath = useMemo(() => {
    if (edges.size === 0) return '';

    const paths: string[] = [];

    // 将边缘转换为线段
    const segments: Array<{ x1: number; y1: number; x2: number; y2: number }> = [];
    
    edges.forEach(edgeKey => {
      const [xStr, yStr, direction] = edgeKey.split(',');
      const x = parseInt(xStr);
      const y = parseInt(yStr);

      if (direction === 'top') {
        segments.push({ x1: x, y1: y, x2: x + 1, y2: y });
      } else if (direction === 'bottom') {
        segments.push({ x1: x, y1: y + 1, x2: x + 1, y2: y + 1 });
      } else if (direction === 'left') {
        segments.push({ x1: x, y1: y, x2: x, y2: y + 1 });
      } else if (direction === 'right') {
        segments.push({ x1: x + 1, y1: y, x2: x + 1, y2: y + 1 });
      }
    });

    // 绘制线段
    segments.forEach((seg, idx) => {
      paths.push(
        `<line x1="${seg.x1 * PIXEL_SIZE}" y1="${seg.y1 * PIXEL_SIZE}" x2="${seg.x2 * PIXEL_SIZE}" y2="${seg.y2 * PIXEL_SIZE}" stroke="#00FFAA" stroke-width="2" stroke-linecap="square"/>`
      );
    });

    return paths.join('');
  }, [edges]);

  const handleClick = (e: React.MouseEvent<HTMLDivElement>, x: number, y: number) => {
    // 阻止事件冒泡，避免触发容器的拖拽和点击处理
    e.stopPropagation();
  };

  return (
    <div
      className="relative"
      style={{
        width: `${GRID_SIZE * PIXEL_SIZE}px`,
        height: `${GRID_SIZE * PIXEL_SIZE}px`,
        backgroundImage: 'linear-gradient(rgba(0, 255, 170, 0.05) 1px, transparent 1px), linear-gradient(90deg, rgba(0, 255, 170, 0.05) 1px, transparent 1px)',
        backgroundSize: `${PIXEL_SIZE}px ${PIXEL_SIZE}px`,
      }}
    >
      {/* SVG 边框层 */}
      {edges.size > 0 && (
        <svg
          className="absolute z-20 pointer-events-none"
          style={{
            left: 0,
            top: 0,
            width: `${GRID_SIZE * PIXEL_SIZE}px`,
            height: `${GRID_SIZE * PIXEL_SIZE}px`,
          }}
          dangerouslySetInnerHTML={{ __html: svgPath }}
        />
      )}

      {/* Render colored cells - 已涂色的像素直接显示颜色，无边框无间隙 */}
      {data?.map(cell => {
        if (!cell.color || cell.color === '#1A1A1A') return null;
        return (
          <div
            key={`${cell.x}-${cell.y}`}
            className="absolute hover:opacity-80 transition-opacity"
            style={{
              left: `${cell.x * PIXEL_SIZE}px`,
              top: `${cell.y * PIXEL_SIZE}px`,
              width: `${PIXEL_SIZE + 0.5}px`,  // 稍微大一点，消除间隙
              height: `${PIXEL_SIZE + 0.5}px`,
              backgroundColor: cell.color,
              outline: 'none',
            }}
          />
        );
      })}
    </div>
  );
}
