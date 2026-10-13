// The titles of the error pages, the same list the api uses.
const TITLES: Record<number, string> = {
  400: 'Bad Request',
  401: 'Unauthorized',
  402: 'Payment Required',
  403: 'Forbidden',
  404: 'Not Found',
  405: 'Method Not Allowed',
  419: 'Page Expired',
  429: 'Too Many Requests',
  503: 'Service Unavailable',
};

export function errorTitle(status: number): string {
  return TITLES[status] ?? 'Server Error';
}

/** What an error page shows: a 403 its own message (the gate's "This
 * action is unauthorized." or the reason it was refused), every other
 * status its title. */
export function errorMessage(status: number, message?: string | null): string {
  if (status === 403 && message) {
    return message;
  }
  return errorTitle(status);
}
