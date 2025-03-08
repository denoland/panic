import { create_symcache, symbolicate } from "./lib/rs_lib.js";

const dSym = new URL("../example/target/debug/example", import.meta.url);
const symcache = create_symcache(Deno.readFileSync(dSym.pathname));

console.log(symcache);

console.log(symbolicate("g4couawkboxb4tbg9oHwqbw6a", symcache));
