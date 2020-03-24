//import {createCheckers} from "ts-interface-checker";

import('../wasm').then(rust => {
  console.log(rust.add(1, 2));
}).catch(console.error);