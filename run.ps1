sbx policy allow network host.docker.internal:5001 `
    --method GET --path '/openapi/public.json'

sbx policy allow network host.docker.internal:5001 `
    --method GET --path '/openapi/private.json'

sbx policy allow network mcp.context7.com:443

sbx run --name rust-rewrite `
    --env OPENAPI_PUBLIC_URL=https://host.docker.internal:5001/openapi/public.json `
    --env OPENAPI_PRIVATE_URL=https://host.docker.internal:5001/openapi/private.json `
    opencode F:\RustProjects\ogma3-tools F:\WebstormProjects\Ogma3.Tools:ro
