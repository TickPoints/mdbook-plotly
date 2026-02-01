// By injecting some functions at the Rust layer, we do further wrapping at that layer.
// So that a more perfect binding effect is achieved.

// This higher level of binding compatibility would implement the following:
// - `console`
// - `window`
// - `window.navigator`

import * as __inter_very_happy_dom__ from "__inter_very_happy_dom__";

const window = __inter_very_happy_dom__.Window.new();
const global = globalThis;
//const console = {
//  log,
//  debug,
//  info,
//  warn,
//  error,
//};