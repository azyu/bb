# API Error Contract

- Keep user-facing errors short and actionable.
- Include HTTP status and endpoint context when helpful.
- Never print tokens, auth headers, or full secret-bearing URLs.
- Preserve machine-parseable output behavior when JSON mode is requested.
