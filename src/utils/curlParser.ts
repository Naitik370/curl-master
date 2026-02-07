export interface ParsedCurl {
    method: string;
    url: string;
    headers: { key: string; value: string; enabled: boolean }[];
    params: { key: string; value: string; enabled: boolean }[];
    body: string;
    bodyType: string;
}

export function parseCurl(curlCommand: string): ParsedCurl {
    const result: ParsedCurl = {
        method: 'GET',
        url: '',
        headers: [],
        params: [],
        body: '',
        bodyType: 'none',
    };

    if (!curlCommand) return result;

    // Normalize command: remove backslashes and combine into one line
    const normalized = curlCommand
        .replace(/\\\n/g, ' ')
        .replace(/\r?\n/g, ' ')
        .trim();

    // Basic regex to find URL (last argument or one after specific flags)
    // This is a naive parser but handles most common cases

    const tokens: string[] = [];
    let currentToken = '';
    let inQuotes = false;
    let quoteChar = '';

    for (let i = 0; i < normalized.length; i++) {
        const char = normalized[i];
        if ((char === '"' || char === "'") && (i === 0 || normalized[i - 1] !== '\\')) {
            if (inQuotes) {
                if (char === quoteChar) {
                    inQuotes = false;
                    tokens.push(currentToken);
                    currentToken = '';
                } else {
                    currentToken += char;
                }
            } else {
                inQuotes = true;
                quoteChar = char;
            }
        } else if (char === ' ' && !inQuotes) {
            if (currentToken) {
                tokens.push(currentToken);
                currentToken = '';
            }
        } else {
            currentToken += char;
        }
    }
    if (currentToken) tokens.push(currentToken);

    for (let i = 0; i < tokens.length; i++) {
        const token = tokens[i];
        const nextToken = tokens[i + 1];

        if (token === '-X' || token === '--request') {
            result.method = nextToken?.toUpperCase() || 'GET';
            i++;
        } else if (token === '-H' || token === '--header') {
            if (nextToken) {
                const parts = nextToken.split(':');
                const key = parts[0]?.trim();
                const value = parts.slice(1).join(':').trim();
                if (key) {
                    result.headers.push({ key, value, enabled: true });
                }
            }
            i++;
        } else if (token === '-d' || token === '--data' || token === '--data-raw' || token === '--data-binary') {
            result.body = nextToken || '';
            result.bodyType = 'raw';
            if (result.method === 'GET') result.method = 'POST'; // curl defaults to POST with data

            // Try to detect JSON
            if (result.body.trim().startsWith('{') || result.body.trim().startsWith('[')) {
                try {
                    JSON.parse(result.body);
                    result.bodyType = 'json';
                } catch (e) {
                    // not valid json
                }
            }
            i++;
        } else if (token.startsWith('http')) {
            result.url = token;
        } else if (i > 0 && !token.startsWith('-') && !tokens[i - 1].startsWith('-')) {
            // Likely the URL if it's a standalone token not preceded by a flag
            if (!result.url) result.url = token;
        }
    }

    // Handle query params from URL
    if (result.url.includes('?')) {
        const [baseUrl, queryStr] = result.url.split('?');
        result.url = baseUrl;
        const searchParams = new URLSearchParams(queryStr);
        searchParams.forEach((value, key) => {
            result.params.push({ key, value, enabled: true });
        });
    }

    return result;
}
