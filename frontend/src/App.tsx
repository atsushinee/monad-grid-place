import { useState, useMemo } from 'react';
import { ConnectButton } from './components/ConnectButton';
import { Grid } from './components/Grid';
import { PixelInfo } from './components/PixelInfo';
import { PaintModal } from './components/PaintModal';
import { GridContainer, Area } from './components/GridContainer';
import { DisplayPixel } from './types';
import { getGridCell } from './services/api';
import { useQueryClient } from '@tanstack/react-query';

function App() {
  const queryClient = useQueryClient();
  const [selectedPixels, setSelectedPixels] = useState<DisplayPixel[]>([]);
  const [isModalOpen, setIsModalOpen] = useState(false);

  const handlePixelSelect = async (x: number, y: number, addToSelection: boolean = false) => {
    if (addToSelection) {
      // 多选模式：直接添加坐标，不请求详情
      setSelectedPixels(prev => {
        const exists = prev.find(p => p.x === x && p.y === y);
        if (exists) {
          return prev.filter(p => p.x !== x || p.y !== y);
        } else {
          return [...prev, { x, y, color: '#1A1A1A' }];
        }
      });
    } else {
      // 单选模式：获取像素详细信息
      try {
        const cell = await getGridCell(x, y);
        setSelectedPixels([{
          x: cell.x,
          y: cell.y,
          color: cell.color || '#1A1A1A',
          owner: cell.owner,
          link: cell.link,
          message: cell.message,
        }]);
      } catch (error) {
        console.error('Failed to fetch pixel info:', error);
        setSelectedPixels([{ x, y, color: '#1A1A1A' }]);
      }
    }
  };

  const handleAreaSelect = (area: Area) => {
    // 框选时添加坐标到现有选择（不清除）
    const newPixels: DisplayPixel[] = [];
    for (let y = area.startY; y <= area.endY; y++) {
      for (let x = area.startX; x <= area.endX; x++) {
        newPixels.push({ x, y, color: '#1A1A1A' });
      }
    }
    
    // 添加到现有选择，去重
    setSelectedPixels(prev => {
      const existing = new Set(prev.map(p => `${p.x},${p.y}`));
      const added = newPixels.filter(p => !existing.has(`${p.x},${p.y}`));
      return [...prev, ...added];
    });
  };

  const handleDeselectPixel = (x: number, y: number) => {
    setSelectedPixels(prev => prev.filter(p => p.x !== x || p.y !== y));
  };

  // 交易成功后刷新网格数据
  const handlePaintSuccess = () => {
    console.log('🔄 Paint successful, refreshing grid data...');
    queryClient.invalidateQueries({ queryKey: ['gridCells'] });
    setSelectedPixels([]); // 清空选择
  };

  const handlePaintClick = () => {
    if (selectedPixels.length > 0) {
      setIsModalOpen(true);
    }
  };

  const handleCloseModal = () => {
    setIsModalOpen(false);
  };

  const selectedPixelForInfo = useMemo(() => {
    if (selectedPixels.length === 0) return null;
    if (selectedPixels.length === 1) return selectedPixels[0];
    // For multi-selection, we can create a summary object
    return {
      x: -1, // Indicates multi-selection
      y: selectedPixels.length,
      color: '#00FFAA',
      owner: 'Multiple',
    };
  }, [selectedPixels]);

  return (
    <div className="bg-[#1A1A1A] text-gray-200 h-screen flex flex-col font-sans overflow-hidden">
      <header className="w-full flex-shrink-0 flex justify-between items-center p-4 border-b border-green-300/10 z-10">
        <h1 className="text-2xl font-bold text-[#00FFAA]">
          GridPlace
        </h1>
        <ConnectButton />
      </header>

      <div className="w-full flex-grow flex relative">
        <main className="flex-grow h-full">
          <GridContainer onPixelSelect={handlePixelSelect} onAreaSelect={handleAreaSelect}>
            <Grid selectedPixels={selectedPixels} onPixelClick={handlePixelSelect} />
          </GridContainer>
        </main>

        {selectedPixelForInfo && (
          <aside className="absolute top-4 right-4 w-72 flex-shrink-0 bg-black/50 backdrop-blur-md rounded-lg z-10 border border-green-300/20">
            <PixelInfo
              selectedPixels={selectedPixels}
              onPaintClick={handlePaintClick}
              onDeselectPixel={handleDeselectPixel}
            />
          </aside>
        )}
      </div>

      {isModalOpen && selectedPixels.length > 0 && (
        <PaintModal
          pixels={selectedPixels}
          onClose={handleCloseModal}
          onSuccess={handlePaintSuccess}
        />
      )}
    </div>
  );
}

export default App;
