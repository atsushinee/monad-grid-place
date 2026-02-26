import { useState } from 'react';
import { ConnectButton } from './components/ConnectButton';
import { Grid } from './components/Grid';
import { PixelInfo } from './components/PixelInfo';
import { PaintModal } from './components/PaintModal';
import { GridContainer } from './components/GridContainer';
import { GridCell } from './types';

export type DisplayPixel = Partial<Omit<GridCell, 'x' | 'y'>> & {
  x: number;
  y: number;
  color: string;
};

function App() {
  const [selectedPixel, setSelectedPixel] = useState<DisplayPixel | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);

  const handlePixelSelect = (pixel: DisplayPixel) => {
    setSelectedPixel(pixel);
  };

  const handlePaintClick = () => {
    if (selectedPixel) {
      setIsModalOpen(true);
    }
  };

  const handleCloseModal = () => {
    setIsModalOpen(false);
  };

  return (
    <div className="bg-gray-900 text-white h-screen flex flex-col font-sans overflow-hidden">
      <header className="w-full flex-shrink-0 flex justify-between items-center p-4 border-b border-cyan-500/20 z-10">
        <h1 className="text-2xl font-bold text-cyan-400">
          GridPlace
        </h1>
        <ConnectButton />
      </header>
      
      <div className="w-full flex-grow flex relative">
        <main className="flex-grow h-full">
          <GridContainer onPixelSelect={handlePixelSelect}>
            <Grid onPixelSelect={handlePixelSelect} />
          </GridContainer>
        </main>
        
        {selectedPixel && (
          <aside className="absolute top-4 right-4 w-72 flex-shrink-0 bg-gray-800/80 backdrop-blur-sm rounded-lg z-10">
            <PixelInfo 
              selectedPixel={selectedPixel} 
              onPaintClick={handlePaintClick} 
            />
          </aside>
        )}
      </div>

      {isModalOpen && selectedPixel && (
        <PaintModal 
          pixel={{ x: selectedPixel.x, y: selectedPixel.y }}
          onClose={handleCloseModal}
        />
      )}
    </div>
  );
}

export default App;
