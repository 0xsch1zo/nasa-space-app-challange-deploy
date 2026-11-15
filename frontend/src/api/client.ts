const TOKEN_KEY = 'astro-health.session';

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
  }
}

let token: string | null = null;
let onUnauthorized: () => void = () => {};

export function setToken(t: string | null) {
  token = t;
}

export function setUnauthorizedHandler(fn: () => void) {
  onUnauthorized = fn;
}

export async function api<T>(path: string, init: RequestInit = {}): Promise<T> {
  const headers = new Headers(init.headers);
  headers.set('Accept', 'application/json');
  if (init.body) headers.set('Content-Type', 'application/json');
  if (token) headers.set('Authorization', `Bearer ${token}`);

  const res = await fetch(`/api${path}`, { ...init, headers });
  if (!res.ok) {
    let message = res.statusText;
    try {
      message = (await res.json()).message ?? message;
    } catch {
      /* non-JSON error body */
    }
    if (res.status === 401 && token) onUnauthorized();
    throw new ApiError(res.status, message);
  }
  return res.json() as Promise<T>;
}

// Session persistence is a convenience only; storage may be unavailable.
export const sessionStore = {
  load<T>(): T | null {
    try {
      const raw = localStorage.getItem(TOKEN_KEY);
      return raw ? (JSON.parse(raw) as T) : null;
    } catch {
      return null;
    }
  },
  save(value: unknown) {
    try {
      localStorage.setItem(TOKEN_KEY, JSON.stringify(value));
    } catch {
      /* ignore */
    }
  },
  clear() {
    try {
      localStorage.removeItem(TOKEN_KEY);
    } catch {
      /* ignore */
    }
  },
};
