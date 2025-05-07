// SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0

import terser from "@rollup/plugin-terser";
import wasm from "@rollup/plugin-wasm";
import replace from "@rollup/plugin-replace";

import fs from "fs/promises";

const appendExportsPlugin = (wasmPath) => {
  return {
    name: "append-exports",
    async transform(code) {
      const wasmBuffer = await fs.readFile(wasmPath);
      const module = await WebAssembly.instantiate(wasmBuffer, {
        env: { extern_rng: () => {} },
      });

      console.log(Object.entries(module.instance.exports));
      const exports = Object.keys(module.instance.exports)
        .map((key) => `export const ${key} = instance.exports.${key};`)
        .join("\n");

      return {
        code: code + exports,
      };
    },
  };
};

export default ({ configWasm }) => {
  if (!configWasm)
    throw Error(
      "please set the location to vodozemac.wasm via --configWasm=FILE",
    );
  if (!configWasm.startsWith("/"))
    throw Error("--configWasm requires an absolute path");

  return {
    input: "src/index.mjs",
    output: {
      file: "dist/vodozemac.mjs",
      format: "esm",
      sourcemap: false,
    },
    plugins: [
      replace({
        preventAssignment: true,
        delimiters: ["'", "'"],
        "vodozemac.wasm": JSON.stringify(configWasm),
      }),
      appendExportsPlugin(configWasm),
      wasm({
        maxFileSize: 1000000000000000,
        targetEnv: "auto-inline",
      }),
      terser(),
    ],
  };
};
