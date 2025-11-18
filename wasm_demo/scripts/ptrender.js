
import init, {add123, addxx,  modify_bytes } from "../pkg/wasm_demo.js";
await init();

function renderPointsWasm(pixels) {
  console.log(addxx(5, 6));
  console.log("==Render Points Wasm==");
  const start = performance.now();
  var num = modify_bytes(pixels);
  const end = performance.now();
  console.log("Took mks:" + num); 
}

window.renderPointsWasm = renderPointsWasm;