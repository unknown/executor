import { EXECUTOR_BASE_URL } from '$env/static/private';
import type { RequestHandler } from './$types';
import { json } from '@sveltejs/kit';

export const GET: RequestHandler = async ({ params }) => {
  const response = await fetch(
    `${EXECUTOR_BASE_URL}/execution-output/${params.job_name}/${params.job_id}`,
    { method: 'GET' }
  );
  return json(await response.json());
};
