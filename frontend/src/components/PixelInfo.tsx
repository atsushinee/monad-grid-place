import { DisplayPixel } from "../App"; // Import the shared type

interface PixelInfoProps {
  selectedPixel: DisplayPixel;
  onPaintClick: () => void;
}

export function PixelInfo({ selectedPixel, onPaintClick }: PixelInfoProps) {
  const hasOwner = selectedPixel.owner && selectedPixel.owner !== '';

  return (
    <div className="w-full h-full p-4">
      <h2 className="text-xl font-bold mb-4 border-b border-gray-700 pb-2">Pixel Info</h2>
      <div className="space-y-3 text-sm">
        <p><strong>Coordinates:</strong> ({selectedPixel.x}, {selectedPixel.y})</p>
        
        <div className="flex items-center">
          <p className="mr-2"><strong>Color:</strong></p>
          <div className="w-5 h-5 rounded border-2 border-gray-500" style={{ backgroundColor: selectedPixel.color }}></div>
          <p className="ml-2 font-mono">{selectedPixel.color}</p>
        </div>
        
        <div>
          <p><strong>Owner:</strong></p>
          <p className="font-mono text-xs break-all text-gray-400">{hasOwner ? selectedPixel.owner : 'None'}</p>
        </div>

        {hasOwner && selectedPixel.link && (
          <div>
            <p><strong>Link:</strong></p>
            <a 
              href={selectedPixel.link} 
              target="_blank" 
              rel="noopener noreferrer"
              className="font-mono text-xs break-all text-cyan-400 hover:underline"
            >
              {selectedPixel.link}
            </a>
          </div>
        )}
      </div>

      <button 
        onClick={onPaintClick}
        className="mt-6 w-full bg-cyan-500 hover:bg-cyan-600 text-white font-bold py-2 px-4 rounded-lg transition duration-300"
      >
        Paint this Pixel
      </button>
    </div>
  );
}
