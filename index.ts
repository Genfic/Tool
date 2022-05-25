import { parse } from 'https://deno.land/std@0.140.0/flags/mod.ts';
import { bundlesize } from './tools/bundlesize/index.ts';

const args = parse(Deno.args);

console.dir(args);

switch (args._[0]) {
	case 'bundlesize':
		bundlesize(args._[1].toString());
		break;
	default:
		console.log('incorrect option');
}
