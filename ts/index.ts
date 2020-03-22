import init, { add } from '../wasm/rs_parts_unknown.js';

async function main() {
  await init();

  console.log(add(1, 2));
}

main();