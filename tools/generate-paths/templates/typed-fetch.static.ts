interface TypedResponse<T> {
	readonly ok: boolean;
	readonly status: number;
	readonly statusText: string;
	readonly headers: Headers;
	readonly data: T;
}

export async function typedFetch<TOut>(
	url: string,
	method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | string,
	body?: object,
	headers?: HeadersInit,
	options?: RequestInit,
): Promise<Readonly<TypedResponse<TOut>>> {
	const res = await fetch(url, {
		method: method,
		headers: {
			"Content-Type": "application/json",
			...headers,
		},
		body: body ? JSON.stringify(body) : null,
		...options,
	});

	const contentType = res.headers.get("content-type");

	let data: TOut;
	if (contentType?.includes("application/json")) {
		data = await res.json();
	} else {
		data = (await res.text()) as TOut;
	}

	return {
		ok: res.ok,
		status: res.status,
		statusText: res.statusText,
		headers: res.headers,
		data: data as TOut,
	};
}
