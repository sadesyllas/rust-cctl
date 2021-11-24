import type { AudioDevices, BluetoothAudioDeviceProfile } from './types';

import { writable } from 'svelte/store';
import { get, post } from '$lib/api';

const API_URL = <string>import.meta.env.VITE_API_URL;

export const devices = writable<AudioDevices>(undefined);

let latestStateTimestamp = 0;

export async function getDevices(): Promise<void> {
  const _devices = await get<AudioDevices>('/audio').catch(() => {
    console.error('Failed to get the available audio devices');

    return <AudioDevices>undefined;
  });

  if (_devices?.timestamp > latestStateTimestamp) {
    latestStateTimestamp = _devices.timestamp;

    devices.set(_devices);
  }
}

export function connectAudioWS(): () => void {
  const url = `ws://${API_URL.replace(/^[^:]+:\/\//, '')}/audio/ws`;
  const ws = new WebSocket(url, []);

  ws.onopen = async () => {
    console.log(`Created a websocket to ${url}`);

    await getDevices();
  };

  ws.onerror = (event) => console.error(`Failed to create a websocket to ${url}: event = ${JSON.stringify(event)}`);

  ws.onclose = (event: CloseEvent) => {
    console.error(`Closed the websocket to ${url}: reason = ${event.reason || '?'}, code = ${event.code}`);

    setTimeout(connectAudioWS, 5_000);
  };

  ws.onmessage = ({ data }: MessageEvent) => {
    const _devices = JSON.parse(data);

    // if (_devices.timestamp > latestStateTimestamp) {
    //   latestStateTimestamp = _devices.timestamp;

    devices.set(_devices);
    // }
  };

  return () => ws.close();
}

export async function setVolume(type: 'source' | 'sink', index: number, volume: number): Promise<void> {
  await post(`${API_URL}/audio/volume`, JSON.stringify({ type, index, volume }));
}

export async function toggleMute(type: 'source' | 'sink', index: number, mute: boolean): Promise<void> {
  await post(`${API_URL}/audio/mute`, JSON.stringify({ type, index, mute }));
}

export async function setProfile(index: number, profile: BluetoothAudioDeviceProfile): Promise<void> {
  await post(`${API_URL}/audio/profile`, JSON.stringify({ index, profile }));
}

export async function setDefault(type: 'source' | 'sink', index: number, name: string): Promise<void> {
  await post(`${API_URL}/audio/default`, JSON.stringify({ type, index, name }));
}
