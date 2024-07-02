import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { EXECUTOR_BASE_URL } from '$env/static/private';

export const POST: RequestHandler = async ({ request }) => {
  const body = await request.json();
  const response = await fetch(`${EXECUTOR_BASE_URL}/execute-rust`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(body)
  });
  return json(await response.json());
};
