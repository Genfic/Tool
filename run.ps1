sbx policy allow network host.docker.internal:5001 `
    --method GET --path '/openapi/public.json'

sbx policy allow network host.docker.internal:5001 `
    --method GET --path '/openapi/internal.json'

sbx policy allow network mcp.context7.com:443
sbx policy allow network localhost:5001

sbx run --name rust-rewrite `
    --env OPENAPI_PUBLIC_URL=https://host.docker.internal:5001/openapi/public.json `
    --env OPENAPI_INTERNAL_URL=https://host.docker.internal:5001/openapi/internal.json `
    opencode F:\RustProjects\ogma3-tools F:\WebstormProjects\Ogma3.Tools:ro
