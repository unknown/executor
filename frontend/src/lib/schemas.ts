import { z } from 'zod';

export const submitSchema = z.discriminatedUnion('status', [
  z.object({
    status: z.literal('Success'),
    job_id: z.string(),
    job_name: z.string()
  }),
  z.object({
    status: z.literal('Error'),
    error: z.string()
  })
]);

export type SubmitResponse = z.infer<typeof submitSchema>;

export const executionOutputSchema = z.discriminatedUnion('status', [
  z.object({
    status: z.literal('Success'),
    output: z.object({
      pending: z.boolean(),
      stdout: z.string(),
      stderr: z.string()
    })
  }),
  z.object({
    status: z.literal('Error'),
    error: z.string()
  })
]);

export type ExecutionOutputResponse = z.infer<typeof executionOutputSchema>;
