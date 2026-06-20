export interface RefreshSingleFlightOptions<Tokens> {
  loadRefreshToken: () => string | null;
  performRefresh: (refreshToken: string) => Promise<Tokens>;
  applySuccess: (tokens: Tokens) => void;
  applyFailure: () => void;
}

export type RefreshSingleFlight<Tokens> = () => Promise<Tokens | null>;

export function createRefreshSingleFlight<Tokens>(
  options: RefreshSingleFlightOptions<Tokens>
): RefreshSingleFlight<Tokens> {
  let inFlight: Promise<Tokens | null> | null = null;

  return function refreshOnce(): Promise<Tokens | null> {
    if (inFlight) {
      return inFlight;
    }

    const refreshToken = options.loadRefreshToken();
    if (!refreshToken) {
      return Promise.resolve(null);
    }

    inFlight = (async () => {
      try {
        const tokens = await options.performRefresh(refreshToken);
        options.applySuccess(tokens);
        return tokens;
      } catch {
        options.applyFailure();
        return null;
      } finally {
        inFlight = null;
      }
    })();

    return inFlight;
  };
}
