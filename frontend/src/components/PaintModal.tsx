import { useState, useEffect } from 'react';
import { useAccount, useWriteContract, useWaitForTransactionReceipt, useReadContract } from 'wagmi';
import { Hex } from 'viem';
import { config } from '../wagmi';
import contractAbi from '../abi/MonadAdWall.json';
import { DisplayPixel, SnapshotRequest } from '../types';
import { generateSnapshot, submitPaintArea } from '../services/api';

interface PaintModalProps {
  pixels: DisplayPixel[];
  onClose: () => void;
  onSuccess?: () => void;
}

const CONTRACT_ADDRESS = '0x5FbDB2315678afecb367f032d93F642f64180aa3';

export function PaintModal({ pixels, onClose }: PaintModalProps) {
  const [color, setColor] = useState('#00FFAA');
  const [link, setLink] = useState('');
  const [message, setMessage] = useState('');
  const [status, setStatus] = useState<'idle' | 'generating' | 'confirming' | 'submitting'>('idle');
  const [totalPrice, setTotalPrice] = useState<bigint>(BigInt(0));

  const { address: accountAddress } = useAccount();
  const isMultiSelect = pixels.length > 1;
  const isConnected = !!accountAddress;

  const { data: hash, writeContractAsync, error: writeError } = useWriteContract({ config });
  const { isLoading: isConfirming, isSuccess: isConfirmed } = useWaitForTransactionReceipt({ hash, config });

  // 将 (x, y) 转换为 index: index = y * 1000 + x
  const indices = pixels.map(p => BigInt(p.y * 1000 + p.x));

  // 从链上读取像素价格
  const { data: prices } = useReadContract({
    address: CONTRACT_ADDRESS,
    abi: contractAbi.abi,
    functionName: 'getPixelPrices',
    args: [indices as any],
    query: {
      enabled: pixels.length > 0,
    },
  });

  // 从链上读取像素所有者
  const { data: owners } = useReadContract({
    address: CONTRACT_ADDRESS,
    abi: contractAbi.abi,
    functionName: 'getPixelOwners',
    args: [indices as any],
    query: {
      enabled: pixels.length > 0,
    },
  });

  // 计算总价格：只计算不属于当前用户的像素（夺取他人像素需要支付）
  useEffect(() => {
    if (prices && Array.isArray(prices) && owners && Array.isArray(owners) && accountAddress) {
      const total = prices.reduce((sum, price, idx) => {
        const owner = (owners[idx] as string).toLowerCase();
        const isOwner = owner === accountAddress.toLowerCase();
        const isEmpty = owner === '0x0000000000000000000000000000000000000000';
        
        // 只有夺取他人的像素才需要支付
        if (!isOwner && !isEmpty) {
          return sum + BigInt(price as any);
        }
        // 自己的像素或空像素：免费
        return sum;
      }, BigInt(0));
      setTotalPrice(total);
    }
  }, [prices, owners, accountAddress]);

  // paintArea 函数不需要检查 cooldown
  // cooldown 只适用于 paint 函数（单个像素竞价模式）
  const canPaint = status === 'idle' && !isConfirming && pixels.length > 0 && link && link !== 'https://' && accountAddress;

  async function handlePaint() {
    if (!accountAddress) {
      alert('Please connect your wallet first!');
      return;
    }
    if (!link || link === 'https://') {
      alert('Please enter a valid link!');
      return;
    }

    setStatus('generating');

    try {
      // 1. 准备快照请求
      const newPixels = pixels.map(p => ({
        x: p.x,
        y: p.y,
        color: color,
        link: link,
        message: message,
      }));

      const snapshotRequest: SnapshotRequest = {
        owner: accountAddress.toLowerCase(),
        new_pixels: newPixels,
      };

      // 2. 调用后端生成快照并上传到 IPFS
      console.log('📦 Generating snapshot...');
      const snapshotResponse = await generateSnapshot(snapshotRequest);
      console.log('✅ Snapshot generated:', snapshotResponse.cid);

      // 3. 提交涂色记录到数据库（可选，用于预先记录）
      await submitPaintArea({
        owner: accountAddress.toLowerCase(),
        cid: snapshotResponse.cid,
        cid_hash: snapshotResponse.cid_hash,
        pixel_count: snapshotResponse.pixel_count,
        total_price: snapshotResponse.total_price,
      });

      // 4. 调用合约 paintArea 函数
      setStatus('confirming');
      console.log('🔗 Calling contract paintArea...');

      const indicesUint256 = pixels.map(p => BigInt(p.y * 1000 + p.x));

      await writeContractAsync({
        address: CONTRACT_ADDRESS,
        abi: contractAbi.abi,
        functionName: 'paintArea',
        args: [indicesUint256 as any, snapshotResponse.cid_hash as Hex],
        value: BigInt(snapshotResponse.total_price),
        account: accountAddress,
      });

      setStatus('submitting');

      // 5. 交易确认后，更新快照记录（带交易哈希）
      if (hash) {
        await submitPaintArea({
          owner: accountAddress.toLowerCase(),
          cid: snapshotResponse.cid,
          cid_hash: snapshotResponse.cid_hash,
          pixel_count: snapshotResponse.pixel_count,
          total_price: snapshotResponse.total_price,
          tx_hash: hash,
        });
      }

      // 6. 通知父组件刷新数据
      if (onSuccess) {
        onSuccess();
      }

      onClose();
    } catch (err) {
      console.error('Paint failed:', err);
      setStatus('idle');
    }
  }

  const getButtonState = () => {
    if (!isConnected) return 'Connect Wallet First';
    if (!link || link === 'https://') return 'Enter Link First';
    if (status === 'generating') return 'Generating Snapshot...';
    if (status === 'confirming') return 'Confirm in Wallet...';
    if (status === 'submitting') return 'Submitting...';
    if (isConfirming) return 'Painting...';
    if (isMultiSelect) return `Paint ${pixels.length} Pixels`;
    return 'Paint';
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-70 flex items-center justify-center z-50 backdrop-blur-sm">
      <div className="bg-[#1A1A1A] border border-green-300/20 p-6 rounded-lg shadow-xl w-full max-w-md">
        <h2 className="text-2xl font-bold mb-4 text-[#00FFAA]">
          {isMultiSelect ? `Paint ${pixels.length} Pixels` : `Paint Pixel (${pixels[0].x}, ${pixels[0].y})`}
        </h2>

        {/* 钱包未连接提示 */}
        {!isConnected && (
          <div className="mb-4 p-3 bg-red-900/50 border border-red-500 rounded-lg">
            <p className="text-red-400 text-sm font-bold">⚠️ Wallet Not Connected</p>
            <p className="text-red-300 text-xs mt-1">Please connect your wallet in the top right corner to proceed.</p>
          </div>
        )}

        {/* 价格信息 */}
        <div className="mb-4 p-3 bg-gray-800 rounded-lg">
          <div className="text-sm text-gray-400">Total Price</div>
          <div className="text-xl font-bold text-[#00FFAA]">
            {(Number(totalPrice) / 1e18).toFixed(4)} ETH
          </div>
          <div className="text-xs text-gray-500">
            {pixels.length} pixels × {(Number(totalPrice) / pixels.length / 1e18).toFixed(4)} ETH
          </div>
        </div>

        <div className="space-y-4">
          <div>
            <label className="block mb-2 text-sm font-medium">Link/Message</label>
            <input
              type="text"
              value={link}
              onChange={(e) => setLink(e.target.value)}
              className="w-full p-2 bg-gray-800 border border-gray-700 rounded-md focus:ring-2 focus:ring-[#00FFAA] focus:border-[#00FFAA]"
              placeholder="https://example.com"
            />
          </div>
          <div>
            <label className="block mb-2 text-sm font-medium">Message (optional)</label>
            <input
              type="text"
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              className="w-full p-2 bg-gray-800 border border-gray-700 rounded-md focus:ring-2 focus:ring-[#00FFAA] focus:border-[#00FFAA]"
              placeholder="Your message here"
            />
          </div>
          <div>
            <label className="block mb-2 text-sm font-medium">Color</label>
            <div className="flex items-center gap-4">
              <input
                type="color"
                value={color}
                onChange={(e) => setColor(e.target.value)}
                className="p-1 h-10 w-14 block bg-gray-800 border border-gray-700 cursor-pointer rounded-lg"
              />
              <input
                type="text"
                value={color}
                onChange={(e) => setColor(e.target.value)}
                className="w-full p-2 bg-gray-800 border border-gray-700 rounded-md font-mono focus:ring-2 focus:ring-[#00FFAA] focus:border-[#00FFAA]"
              />
            </div>
          </div>
        </div>

        <div className="flex justify-end gap-4 mt-6">
          <button
            onClick={onClose}
            disabled={isConfirming}
            className="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded-lg disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            onClick={handlePaint}
            disabled={!canPaint || !isConnected}
            title={!isConnected ? 'Please connect wallet first' : undefined}
            className="bg-[#00FFAA] hover:bg-green-400 text-black font-bold py-2 px-4 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
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
