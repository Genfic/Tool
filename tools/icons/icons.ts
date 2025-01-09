import { Eta } from "@eta-dev/eta";
import * as path from "@std/path";

const __dirname = path.dirname(path.fromFileUrl(import.meta.url));
const eta = new Eta({
	views: path.join(__dirname, "templates"),
	autoEscape: false,
	autoFilter: true,
	filterFunction: (value): string => {
		if (value === undefined || value === null) return "";
		return value.toString();
	},
});

export const addIcons = async (name: string) => {
	const res = await fetch(
		`https://unpkg.com/lucide-static@latest/icons/${name}.svg`,
	);
	if (!res.ok) {
		console.log(`Could not download icon: ${res.status} ${res.statusText}`);
		return;
	}
	const icon = await res.text();
	console.log(icon);
};
