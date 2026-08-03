import { describe, expect, it } from 'vitest';

import { getCliToolDefinitions } from '../../src/server.js';

describe('MCP tool schemas', () => {
   it('exposes object input schemas for every tool', () => {
      const definitions = getCliToolDefinitions();

      expect(definitions).toHaveLength(22);

      for (const definition of definitions) {
         expect(definition.inputSchema.type).toBe('object');
      }
   });

   it('preserves all native dialog interaction alternatives', () => {
      const definition = getCliToolDefinitions().find((tool) => { return tool.name === 'native_dialog_interact'; });

      expect(definition).toBeDefined();

      const alternatives = definition?.inputSchema.anyOf;

      expect(Array.isArray(alternatives)).toBe(true);
      expect(alternatives).toHaveLength(4);
   });
});
