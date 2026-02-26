import { useState, useMemo } from 'react';
import { useAccount, useWriteContract, useWaitForTransactionReceipt, useReadContract } from 'wagmi';
import { formatEther, Hex } from 'viem';
import { config } from '../wagmi';
import contractAbi from '../abi/MonadAdWall.json';
import { uploadMetadata, cacheCid } from '../services/api';

interface PaintModalProps {
  pixel: { x: number; y: number; };
  onClose: () => void;
}

const CONTRACT_ADDRESS = '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512';

type PixelStruct = {
  price: bigint;
  expiry: bigint;
};

export function PaintModal({ pixel, onClose }: PaintModalProps) {
  const [color, setColor] = useState('#FFFFFF');
  const [link, setLink] = useState('https://');
  const [status, setStatus] = useState<'idle' | 'uploading' | 'caching' | 'confirming' | 'painting'>('idle');
  
  const { address: accountAddress } = useAccount();
  const index = useMemo(() => BigInt(pixel.y * 1000 + pixel.x), [pixel]);

  const { data: pixelData, status: pixelDataStatus } = useReadContract({
    address: CONTRACT_ADDRESS,
    abi: contractAbi.abi,
    functionName: 'getPixel',
    args: [index],
    config,
  });

  const { data: initialPrice, status: initialPriceStatus } = useReadContract({
    address: CONTRACT_ADDRESS,
    abi: contractAbi.abi,
    functionName: 'INITIAL_PRICE',
    config,
  });

  const currentPrice = useMemo(() => {
    if (pixelData === undefined || initialPrice === undefined) return undefined;
    const typedPixelData = pixelData as PixelStruct;
    const onChainPrice = typedPixelData.price;
    const expiry = typedPixelData.expiry;
    if (onChainPrice === undefined) return undefined;

    const isUnpainted = onChainPrice === 0n;
    const isExpired = !isUnpainted && Number(expiry) > 0 && Number(expiry) < (Date.now() / 1000);

    if (isUnpainted || isExpired) {
      return initialPrice as bigint; // Assert type here
    }
    return onChainPrice;
  }, [pixelData, initialPrice]);

  const { data: hash, writeContractAsync, error: writeError } = useWriteContract({config});
  const { isLoading: isConfirming, isSuccess: isConfirmed } = useWaitForTransactionReceipt({ hash, config });

  async function handlePaint() {
    if (!link || currentPrice === undefined) return;

    setStatus('uploading');
    let cid: string;
    let cidHash: Hex;
    try {
      const response = await uploadMetadata({ link, message: '' });
      cid = response.cid;
      cidHash = response.cid_hash as Hex;
    } catch (e) {
      alert("Failed to upload metadata.");
      console.error(e);
      setStatus('idle');
      return;
    }

    setStatus('caching');
    try {
      await cacheCid({ cid_hash: cidHash, cid });
    } catch (e) {
      alert("Failed to cache metadata.");
      console.error(e);
      setStatus('idle');
      return;
    }

    setStatus('confirming');
    try {
      await writeContractAsync({
        address: CONTRACT_ADDRESS,
        abi: contractAbi.abi,
        functionName: 'paint',
        args: [index, parseInt(color.substring(1), 16), cidHash],
        value: currentPrice,
        account: accountAddress,
      });
    } catch (err) {
      console.error("Transaction failed:", err);
      setStatus('idle');
    }
  }

  const priceInEth = currentPrice !== undefined ? formatEther(currentPrice) : '...';
  const isProcessing = status !== 'idle' || isConfirming;

  const getButtonState = () => {
    if (pixelDataStatus === 'pending' || initialPriceStatus === 'pending') return "Preparing...";
    if (status === 'uploading') return "Uploading...";
    if (status === 'caching') return "Caching...";
    if (status === 'confirming') return "Confirm in Wallet...";
    if (isConfirming) return "Painting...";
    return `Paint (${priceInEth} ETH)`;
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-70 flex items-center justify-center z-50">
      <div className="bg-gray-800 p-6 rounded-lg shadow-xl w-full max-w-md">
        <h2 className="text-2xl font-bold mb-4">Paint Pixel ({pixel.x}, {pixel.y})</h2>
        <div className="space-y-4">
          <div>
            <label className="block mb-2 text-sm font-medium">Link/Message</label>
            <input type="text" value={link} onChange={(e) => setLink(e.target.value)} className="w-full p-2 bg-gray-700 border border-gray-600 rounded-md"/>
          </div>
          <div>
            <label className="block mb-2 text-sm font-medium">Color</label>
            <div className="flex items-center gap-4">
              <input type="color" value={color} onChange={(e) => setColor(e.target.value)} className="p-1 h-10 w-14 block bg-gray-700 border border-gray-600 cursor-pointer rounded-lg"/>
              <input type="text" value={color} onChange={(e) => setColor(e.target.value)} className="w-full p-2 bg-gray-700 border border-gray-600 rounded-md font-mono"/>
            </div>
          </div>
        </div>
        <div className="flex justify-end gap-4 mt-6">
          <button onClick={onClose} className="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded-lg">Cancel</button>
          <button 
            onClick={handlePaint} 
            disabled={currentPrice === undefined || isProcessing} 
            className="bg-cyan-500 hover:bg-cyan-600 text-white font-bold py-2 px-4 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {getButtonState()}
          </button>
        </div>
        {hash && <div className="mt-4 text-xs break-all">Tx Hash: {hash}</div>}
        {isConfirming && <div className="mt-2 text-sm text-yellow-400">Waiting for transaction...</div>}
        {isConfirmed && <div className="mt-2 text-sm text-green-400">Success! Grid will update.</div>}
        {writeError && <div className="mt-2 text-sm text-red-500">Error: {writeError.message}</div>}
      </div>
    </div>
  );
}
