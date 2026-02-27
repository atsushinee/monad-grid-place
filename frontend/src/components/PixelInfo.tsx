import { DisplayPixel } from "../types";

interface PixelInfoProps {
  selectedPixels: DisplayPixel[];
  onPaintClick: () => void;
  onDeselectPixel?: (x: number, y: number) => void;
}

export function PixelInfo({ selectedPixels, onPaintClick, onDeselectPixel }: PixelInfoProps) {
  if (selectedPixels.length === 0) {
    return null;
  }

  const isMultiSelect = selectedPixels.length > 1;
  const pixel = selectedPixels[0];

  // 计算选择区域的边界
  const bounds = selectedPixels.length > 0 ? {
    minX: Math.min(...selectedPixels.map(p => p.x)),
    maxX: Math.max(...selectedPixels.map(p => p.x)),
    minY: Math.min(...selectedPixels.map(p => p.y)),
    maxY: Math.max(...selectedPixels.map(p => p.y)),
  } : null;

  return (
    <div className="w-full h-full p-4 overflow-y-auto">
      <h2 className="text-xl font-bold mb-4 border-b border-green-300/20 pb-2 text-[#00FFAA]">
        {isMultiSelect ? "Selection Info" : "Pixel Info"}
      </h2>

      {isMultiSelect ? (
        <div className="space-y-3 text-sm">
          <p><strong>Pixels Selected:</strong> <span className="text-[#00FFAA]">{selectedPixels.length}</span></p>
          
          {bounds && (
            <div className="p-3 bg-gray-800 rounded-lg">
              <p className="text-xs text-gray-400 mb-2">Area bounds:</p>
              <p className="font-mono text-xs">
                ({bounds.minX}, {bounds.minY}) → ({bounds.maxX}, {bounds.maxY})
              </p>
            </div>
          )}

          <div className="p-3 bg-gray-800 rounded-lg">
            <p className="text-xs text-gray-400 mb-2">Selected pixels:</p>
            <div className="max-h-48 overflow-y-auto space-y-1">
              {selectedPixels.map((p, idx) => (
                <div key={`${p.x}-${p.y}-${idx}`} className="flex items-center justify-between text-xs font-mono hover:bg-gray-700 p-1 rounded">
                  <span>({p.x}, {p.y})</span>
                  {onDeselectPixel && (
                    <button
                      onClick={() => onDeselectPixel(p.x, p.y)}
                      className="text-red-400 hover:text-red-300 px-2"
                    >
                      ✕
                    </button>
                  )}
                </div>
              ))}
            </div>
          </div>

          <p className="text-xs text-gray-400">
            Hold <kbd className="px-2 py-1 bg-gray-700 rounded">Ctrl</kbd> + Click to add/remove individual pixels.
          </p>
        </div>
      ) : (
        <div className="space-y-3 text-sm">
          <p><strong>Coordinates:</strong> ({pixel.x}, {pixel.y})</p>
          <div className="flex items-center">
            <p className="mr-2"><strong>Color:</strong></p>
            <div className="w-5 h-5 rounded border-2 border-gray-500" style={{ backgroundColor: pixel.color }}></div>
            <p className="ml-2 font-mono">{pixel.color}</p>
          </div>
          {pixel.owner && (
            <div>
              <p><strong>Owner:</strong></p>
              <p className="font-mono text-xs break-all text-gray-400">{pixel.owner}</p>
            </div>
          )}
          {pixel.link && (
            <div>
              <p><strong>Link:</strong></p>
              <a 
                href={pixel.link} 
                target="_blank" 
                rel="noopener noreferrer" 
                className="font-mono text-xs break-all text-green-400 hover:underline block max-w-full truncate"
              >
                {pixel.link}
              </a>
            </div>
          )}
          {pixel.message && (
            <div>
              <p><strong>Message:</strong></p>
              <p className="font-mono text-xs break-all text-gray-300">{pixel.message}</p>
            </div>
          )}
          {pixel.extraData && Object.keys(pixel.extraData).length > 0 && (
            <div>
              <p><strong>Extra Data:</strong></p>
              <pre className="font-mono text-xs bg-gray-800 p-2 rounded overflow-auto max-h-32">
                {JSON.stringify(pixel.extraData, null, 2)}
              </pre>
            </div>
          )}
        </div>
      )}

      <button
        onClick={(e) => {
          e.stopPropagation();
          console.log('Paint button clicked, selectedPixels:', selectedPixels.length);
          onPaintClick();
        }}
        className="mt-6 w-full bg-[#00FFAA] hover:bg-green-400 text-black font-bold py-2 px-4 rounded-lg transition duration-300"
      >
        {isMultiSelect ? `Paint All (${selectedPixels.length})` : "Paint this Pixel"}
      </button>
    </div>
  );
}
