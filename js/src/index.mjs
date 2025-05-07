// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

import wasm from "vodozemac.wasm";

const { instance } = await wasm({
  env: {
    extern_rng: (ptr, len) => {
      const memory = new Uint8Array(instance.exports.memory.buffer, ptr, len);
      crypto.getRandomValues(memory);
      return 0;
    },
  },
});

export { instance };
