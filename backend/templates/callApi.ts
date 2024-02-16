export default async function callApi(endpoint: string, params: any): Promise<Response> {
    try {
        const token = localStorage.getItem('jwt');
        const response = await fetch(`http://localhost:3030/api/${endpoint}`, {
            method: 'POST',
            headers: {
                Authorization: token || undefined,
                'Content-Type': 'application/json',
            } as HeadersInit,
            body: JSON.stringify(params),
        });
        if (!response.ok) throw new Error('API request failed');
        return response
    } catch (error) {
        if (error instanceof Error) {
            throw new Error(`API request failed: ${error.message}`);
        } else {
            throw new Error('API request failed: Unknown error');
        }
    }
}