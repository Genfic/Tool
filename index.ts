import {parse} from 'https://deno.land/std@0.144.0/flags/mod.ts';
import {bundlesize} from './tools/bundlesize/index.ts';
import {generateImports} from './tools/generate-imports/index.ts';
import {generatePaths} from './tools/generate-paths/generate-paths.ts';
import {generatePreloads} from './tools/generate-preloads/index.ts';
import {generateManifest} from './tools/generate-js-manifest/index.ts';

const args = parse(Deno.args, {collect: ['path']});

const [command, ...params] = args._;
const run = async () => {
    switch (command) {
        case 'bundlesize':
            return await bundlesize(params.map((x) => x.toString()));
        case 'generate-imports':
            return await generateImports(params[0].toString(), params[1].toString());
        case 'generate-preloads':
            return await generatePreloads(params[0].toString(), params[1].toString());
        case 'generate-paths':
            return await generatePaths(params[0].toString(), args['path']);
        case 'generate-js-manifest':
            return await generateManifest(params[0].toString(), params.at(1)?.toString());
        case 'demo':
            return await demo({command, params, args});
        default:
            console.log(`[${command}] is not a correct option`);
    }
};
await run();

async function demo(obj: object) {
    await Promise.resolve();
    console.info(JSON.stringify(obj));
}
