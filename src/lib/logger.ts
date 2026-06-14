import { invoke } from '@tauri-apps/api/core';

export type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error';

export async function logMessage(
    level: LogLevel,
    module: string,
    message: string,
): Promise<void> {
    try {
        await invoke('log_message', { level, module, message });
    } catch {
    }
}
