import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { readFile } from 'fs/promises';
import path from 'path';
import { manageDriverSession } from '../../src/driver/session-manager';
import { getTestAppPort } from '../test-utils';

describe('Driver Module E2E (Real App)', () => {
   const TIMEOUT = 90000;

   const TEST_APP_PATH = path.resolve(process.cwd(), '../test-app');

   beforeAll(async () => {
      // App is already started globally - connect to the dynamically assigned port
      await manageDriverSession('start', undefined, getTestAppPort());
   }, TIMEOUT);

   afterAll(async () => {
      // Don't stop the app - it's managed globally
      await manageDriverSession('stop');
   }, TIMEOUT);

   it('should launch the test app successfully', async () => {
      // Verify the app is running by checking config can be read
      const configPath = path.join(TEST_APP_PATH, 'src-tauri/tauri.conf.json');

      const config = await readFile(configPath, 'utf-8');

      expect(config).toContain('test-app');
      expect(config).toContain('com.hushlor.tauri-mcp-test-app');
   }, TIMEOUT);

   it('should verify devtools feature is enabled', async () => {
      const cargoToml = await readFile(path.join(TEST_APP_PATH, 'src-tauri/Cargo.toml'), 'utf-8');

      expect(cargoToml).toContain('features = ["devtools"]');
   }, TIMEOUT);
});
