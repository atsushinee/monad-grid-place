import axios from 'axios';
import { GridCell } from '../types';

const apiClient = axios.create({
  baseURL: 'http://127.0.0.1:3000',
});

export const getGridCells = async (page: number = 1): Promise<GridCell[]> => {
  const response = await apiClient.get(`/grid?page=${page}`);
  return response.data;
};

interface UploadPayload {
  link: string;
  message: string;
}

interface UploadResponse {
  cid: string;
  cid_hash: string;
}

export const uploadMetadata = async (payload: UploadPayload): Promise<UploadResponse> => {
  const response = await apiClient.post('/upload', payload);
  return response.data;
}

interface CachePayload {
  cid_hash: string;
  cid: string;
}

export const cacheCid = async (payload: CachePayload): Promise<void> => {
  await apiClient.post('/cache', payload);
}
