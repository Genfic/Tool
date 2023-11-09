import {WalkEntry} from "https://deno.land/std@0.143.0/fs/walk.ts";

export interface FileData {
	name: string;
	size: number;
}

export interface PathData {
	dir: string;
	name: string;
	path: string;
}
