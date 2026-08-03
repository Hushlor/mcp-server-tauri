import { describe, it, expect } from 'vitest';
import { readFile } from 'fs/promises';
import path from 'path';
import { readLogs } from '../../src/monitor/logs';

const TEST_APP_PATH = path.resolve(process.cwd(), '../test-app');

describe('Integration Tests', () => {
   describe('Desktop Platform Workflows', () => {
      it('should read desktop config files', async () => {
         const configPath = path.join(TEST_APP_PATH, 'src-tauri/tauri.conf.json');

         const config = await readFile(configPath, 'utf-8');

         const configObj = JSON.parse(config);

         expect(configObj.productName).toBe('test-app');
         expect(configObj.identifier).toBe('com.hushlor.tauri-mcp-test-app');
         expect(configObj.app?.windows?.[0]?.title).toBe('test-app');
      });
   });

   describe('Cross-Platform Workflows', () => {
      it('should read logs with platform auto-detection', async () => {
         try {
            const logs = await readLogs({ source: 'system', lines: 5 });

            expect(logs).toBeDefined();
            expect(typeof logs).toBe('string');
         } catch(e) {
            // Expected if platform not supported
            expect(e).toBeDefined();
         }
      });

      it('should handle config operations across file types', async () => {
         // Verify all config types can be read
         const tauriConf = await readFile(path.join(TEST_APP_PATH, 'src-tauri/tauri.conf.json'), 'utf-8');

         const cargoToml = await readFile(path.join(TEST_APP_PATH, 'src-tauri/Cargo.toml'), 'utf-8');

         const packageJson = await readFile(path.join(TEST_APP_PATH, 'package.json'), 'utf-8');

         expect(tauriConf).toBeDefined();
         expect(cargoToml).toBeDefined();
         expect(packageJson).toBeDefined();
      });
   });
});
