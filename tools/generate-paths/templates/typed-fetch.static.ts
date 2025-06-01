type SuccessResponse<TData> = {
	readonly ok: true;
	readonly status: number;
	readonly statusText: string;
	readonly headers: Headers;
	readonly data: TData;
};

type ErrorResponse<TError> = {
	readonly ok: false;
	readonly status: number;
	readonly statusText: string;
	readonly headers: Headers;
	readonly error: TError;
};

type TypedResponse<TData, TError> =
	| SuccessResponse<TData>
	| ErrorResponse<TError>;

type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD";

type ResponseType = {
	[K in keyof Response]: Response[K] extends (
		...args: unknown[]
	) => Promise<unknown> ? K
		: never;
}[keyof Response];

type CustomString = string & { ___?: never };

type RequestOptions = Omit<RequestInit, "method" | "headers" | "body"> & {
	responseType?: ResponseType;
	ignoreErrors?: boolean;
};

export const get: HttpMethod = "GET";
export const post: HttpMethod = "POST";
export const put: HttpMethod = "PUT";
export const patch: HttpMethod = "PATCH";
export const del: HttpMethod = "DELETE";
export const head: HttpMethod = "HEAD";

export type KnownHeaders =
	| "Accept"
	| "Accept-Charset"
	| "Accept-Encoding"
	| "Accept-Language"
	| "Authorization"
	| "Cache-Control"
	| "Content-Length"
	| "Content-Type"
	| "Cookie"
	| "ETag"
	| "Forwarded"
	| "From"
	| "Host"
	| "If-Match"
	| "If-Modified-Since"
	| "If-None-Match"
	| "If-Range"
	| "If-Unmodified-Since"
	| "Max-Forwards"
	| "Origin"
	| "Range"
	| "Referer"
	| "User-Agent";

export async function typedFetch<TOut, TBody>(
	url: string,
	method: HttpMethod | CustomString,
	body?: TBody,
	headers?: Record<KnownHeaders | CustomString, string>,
	options?: RequestOptions,
): Promise<TypedResponse<TOut, string>> {
	try {
		const res = await fetch(url, {
			method: method,
			headers: {
				"content-type": "application/json",
				...headers,
			},
			body: body && JSON.stringify(body),
			...options,
		});

		if (res.status >= 400 && !options?.ignoreErrors) {
			return {
				ok: false,
				status: res.status,
				statusText: res.statusText,
				headers: res.headers,
				error: await res.text().catch(() => res.statusText),
			};
		}
		const contentType = res.headers.get("Content-Type")?.toLowerCase() || "";

		const data = options?.responseType ? await res[options.responseType]() : await (() => {
			if (contentType.includes("application/json")) {
				return res.json();
			}
			if (/^(application|image|audio|video)\//.test(contentType)) {
				return res.blob();
			}
			return res.text();
		})();

		return {
			ok: true,
			status: res.status,
			statusText: res.statusText,
			headers: res.headers,
			data: data,
		};
	} catch (e) {
		return {
			ok: false,
			status: 0,
			statusText: "",
			headers: new Headers(),
			error: e instanceof Error ? e.message : String(e),
		};
	}
}
