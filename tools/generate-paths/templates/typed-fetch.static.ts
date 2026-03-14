export type TypedResponse<TResponses extends Record<number, unknown>> = {
    [S in keyof TResponses & number]: {
        readonly ok: `${S}` extends `2${string}` ? true : false;
        readonly status: S;
        readonly statusText: string;
        readonly headers: Headers;
        readonly data: TResponses[S];
    }
}[keyof TResponses & number];

export type ErrorTypedResponse = {
    readonly ok: false;
    readonly status: 0;
    readonly statusText: string;
    readonly headers: Headers;
    readonly data: undefined;
    readonly error: string;
};

type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD";

type ResponseType = {
    [K in keyof Response]: Response[K] extends (
            ...args: unknown[]
        ) => Promise<unknown> ? K
        : never;
}[keyof Response];

type CustomString = string & { ___?: never };

export type RequestOptions = Omit<RequestInit, "method" | "headers" | "body"> & {
    responseType?: ResponseType;
    ignoreErrors?: boolean;
};

export const GET: HttpMethod = "GET";
export const POST: HttpMethod = "POST";
export const PUT: HttpMethod = "PUT";
export const PATCH: HttpMethod = "PATCH";
export const DELETE: HttpMethod = "DELETE";
export const HEAD: HttpMethod = "HEAD";

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
    | "If-Match"
    | "If-Modified-Since"
    | "If-None-Match"
    | "If-Unmodified-Since"
    | "Origin"
    | "Referer"
    | "User-Agent";

const ISO_8601 = /^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:?\d{2})?)?$/;

export const DateSafeJsonParse = <T>(text: string): T => JSON.parse(text, (_, value) => {
    if (typeof value === 'string' && ISO_8601.test(value)) {
        const date = new Date(value);
        if (!Number.isNaN(date.getTime())) return date;
    }
    return value as T;
})

export async function typedFetch<TResponses extends Record<number, unknown>, TBody>(
    url: string,
    method: HttpMethod | CustomString,
    body?: TBody,
    headers?: HeadersInit,
    options?: RequestOptions,
): Promise<TypedResponse<TResponses> | ErrorTypedResponse> {
    try {
        const res = await fetch(url, {
            method: method as string,
            headers: {
                "content-type": "application/json",
                ...headers,
            },
            body: body && JSON.stringify(body),
            ...options,
        } as RequestInit);

        const contentType = res.headers.get("Content-Type")?.toLowerCase() ?? "";

        let data: unknown;

        if (options?.responseType) {
            data = await res[options.responseType]();
        } else if (contentType.includes("application/json")) {
            const text = await res.text();
            data = DateSafeJsonParse(text);
        } else if (/^(application|image|audio|video)\//.test(contentType)) {
            data = await res.blob();
        } else {
            data = await res.text();
        }

        return {
            ok: res.ok,
            status: res.status,
            statusText: res.statusText,
            headers: res.headers,
            data: data,
        } as unknown as TypedResponse<TResponses>;;
    } catch (e) {
        return {
            ok: false,
            status: 0,
            statusText: "",
            headers: new Headers(),
            error: e instanceof Error ? e.message : String(e),
        } as unknown as ErrorTypedResponse;
    }
}
