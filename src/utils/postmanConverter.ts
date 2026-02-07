export interface PostmanHeader {
    key: string;
    value: string;
    disabled?: boolean;
}

export interface PostmanRequest {
    method: string;
    header: PostmanHeader[];
    body?: {
        mode: string;
        raw?: string;
        urlencoded?: Array<{ key: string; value: string }>;
        formdata?: Array<{ key: string; value: string }>;
    };
    url: {
        raw: string;
        host?: string[];
        path?: string[];
        query?: Array<{ key: string; value: string; disabled?: boolean }>;
    };
    auth?: {
        type: string;
        bearer?: Array<{ key: string; value: string }>;
        basic?: Array<{ key: string; value: string }>;
    };
}

export interface PostmanItem {
    name: string;
    request?: PostmanRequest;
    item?: PostmanItem[];
}

export interface PostmanCollection {
    info: {
        name: string;
        schema: string;
    };
    item: PostmanItem[];
}

// Internal Representation for Import
export interface CMRequest {
    name: string;
    method: string;
    url: string;
    headers: string; // JSON string for DB
    params: string; // JSON string for DB
    body: string; // JSON string for DB
    auth: string; // JSON string for DB
}

export interface CMFolder {
    name: string;
    requests: CMRequest[];
    folders: CMFolder[];
}

export interface CMCollection {
    name: string;
    workspace_id?: string;
    folders: CMFolder[];
    requests: CMRequest[];
}

export function convertPostmanToCM(collectionData: PostmanCollection): CMCollection {
    const collection: CMCollection = {
        name: collectionData.info.name || 'Imported Postman Collection',
        folders: [],
        requests: []
    };

    function processItems(items: PostmanItem[], targetFolders: CMFolder[], targetRequests: CMRequest[]) {
        for (const item of items) {
            if (item.item) {
                // It's a folder
                const folder: CMFolder = {
                    name: item.name,
                    requests: [],
                    folders: []
                };
                processItems(item.item, folder.folders, folder.requests);
                targetFolders.push(folder);
            } else if (item.request) {
                // It's a request
                targetRequests.push(convertRequest(item));
            }
        }
    }

    function convertRequest(item: PostmanItem): CMRequest {
        const pReq = item.request!;

        // URL and Params
        let url = '';
        const params: any[] = [];

        if (typeof pReq.url === 'string') {
            url = pReq.url;
        } else if (pReq.url && pReq.url.raw) {
            url = pReq.url.raw.split('?')[0]; // Use base URL
            if (pReq.url.query) {
                for (const q of pReq.url.query) {
                    params.push({
                        key: q.key,
                        value: q.value,
                        enabled: !q.disabled
                    });
                }
            }
        }

        // Headers
        const headers = (pReq.header || []).map(h => ({
            key: h.key,
            value: h.value,
            enabled: !h.disabled
        }));

        // Body
        let bodyData = { type: 'none', content: '' };
        if (pReq.body) {
            if (pReq.body.mode === 'raw') {
                bodyData = {
                    type: pReq.body.raw?.trim().startsWith('{') ? 'json' : 'raw',
                    content: pReq.body.raw || ''
                };
            } else if (pReq.body.mode === 'formdata') {
                // Simplified mapping for now
                bodyData = { type: 'raw', content: JSON.stringify(pReq.body.formdata || []) };
            }
        }

        // Auth
        let authData: any = { type: 'none' };
        if (pReq.auth) {
            if (pReq.auth.type === 'bearer' && pReq.auth.bearer) {
                const token = pReq.auth.bearer.find(b => b.key === 'token')?.value || '';
                authData = { type: 'bearer', token };
            } else if (pReq.auth.type === 'basic' && pReq.auth.basic) {
                const username = pReq.auth.basic.find(b => b.key === 'username')?.value || '';
                const password = pReq.auth.basic.find(b => b.key === 'password')?.value || '';
                authData = { type: 'basic', username, password };
            }
        }

        return {
            name: item.name,
            method: pReq.method || 'GET',
            url: url,
            headers: JSON.stringify(headers),
            params: JSON.stringify(params),
            body: JSON.stringify(bodyData),
            auth: JSON.stringify(authData)
        };
    }

    processItems(collectionData.item, collection.folders, collection.requests);

    return collection;
}
