/** Badge variant per PrimeResult keyword (`success` | `failed` | `hold` | `skip`). */
export const RESULT_VARIANT: Record<
  string,
  'success' | 'destructive' | 'secondary' | 'warning'
> = {
  success: 'success',
  failed: 'destructive',
  hold: 'warning',
  skip: 'secondary',
}
