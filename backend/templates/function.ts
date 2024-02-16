import callApi from './callApi';

export async function $function_name$(params: $arg_type$): Promise<$return_type$> {
    const response = await callApi($route$, params);
    return (await response.json()) as Promise<$return_type$>;
}