import { z } from 'zod';
import { execa } from 'execa';

export const ListDevicesSchema = z.object({});

const DEVICE_LIST_TIMEOUT_MS = 5000;

async function getAndroidDevices(): Promise<string[]> {
   try {
      const { stdout } = await execa('adb', [ 'devices', '-l' ], { timeout: DEVICE_LIST_TIMEOUT_MS });

      return stdout
         .split('\n')
         .slice(1)
         .filter((line) => { return line.trim().length > 0; })
         .map((line) => { return line.trim(); });
   } catch(_) {
      // Android SDK not available or adb command failed
      return [];
   }
}

async function getIOSSimulators(): Promise<string[]> {
   if (process.platform !== 'darwin') {
      return [];
   }

   try {
      const { stdout } = await execa(
         'xcrun',
         [ 'simctl', 'list', 'devices', 'booted' ],
         { timeout: DEVICE_LIST_TIMEOUT_MS }
      );

      return stdout
         .split('\n')
         .filter((line) => { return line.trim().length > 0 && !line.includes('== Devices =='); });
   } catch(_) {
      // Xcode not installed or xcrun command failed
      return [];
   }
}

export async function listDevices(): Promise<{ android: string[]; ios: string[] }> {
   const [ android, ios ] = await Promise.all([
      getAndroidDevices(),
      getIOSSimulators(),
   ]);

   return { android, ios };
}
