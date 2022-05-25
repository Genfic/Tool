import { parse } from "https://deno.land/std@0.140.0/flags/mod.ts";
import { bundlesize } from "./tools/bundlesize/index.ts";
import { generateImports } from "./tools/generate-imports/index.ts";
import { generatePreloads } from "./tools/generate-preloads/index.ts";

const args = parse(Deno.args);

const [command, ...params] = args._;
const run = async () => {
  switch (command) {
    case "bundlesize":
      return await bundlesize(params.map((x) => x.toString()));
    case "generate-imports":
      return await generateImports(params[0].toString(), params[1].toString());
    case "generate-preloads":
      return await generatePreloads(params[0].toString(), params[1].toString());
    default:
      console.log("incorrect option");
  }
};
await run();
