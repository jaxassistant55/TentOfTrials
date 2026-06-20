/**
 * @fileoverview API configuration with explicit environment validation.
 *
 * This module replaces the implicit localhost fallback with explicit
 * environment validation. In development, it uses a documented default.
 * In production, it requires explicit configuration and fails fast if missing.
 */

/**
 * Environment interface for dependency injection (primarily for testing).
 */
export interface Environment {
  mode?: string;
  apiBaseUrl?: string;
}

/**
 * Gets the current environment from import.meta.env or process.env.
 */
export function getCurrentEnvironment(): Environment {
  // Vite environment
  if (typeof import.meta !== 'undefined' && import.meta.env) {
    return {
      mode: import.meta.env.MODE,
      apiBaseUrl: import.meta.env.VITE_API_BASE_URL,
    };
  }
  // Node/fallback environment
  if (typeof process !== 'undefined' && process.env) {
    return {
      mode: process.env.NODE_ENV,
      apiBaseUrl: process.env.VITE_API_BASE_URL,
    };
  }
  return {};
}

/**
 * Determines if the given environment is development.
 */
export function isDevelopment(env: Environment): boolean {
  return env.mode === 'development';
}

/**
 * Normalizes a base URL by removing trailing slashes.
 * This prevents double-slash issues when constructing full URLs.
 */
export function normalizeBaseUrl(url: string): string {
  return url.replace(/\/+$/, '');
}

/**
 * Gets the API base URL with explicit environment validation.
 *
 * Development behavior:
 * - Uses /api/v1 as the default base URL
 * - This routes through Vite's dev server proxy to localhost:8080
 * - Can be overridden with VITE_API_BASE_URL for testing against other backends
 *
 * Production behavior:
 * - REQUIRES VITE_API_BASE_URL environment variable
 * - Throws an error if the variable is missing or empty
 * - This ensures production builds fail fast with a clear error message
 *
 * @param env Optional environment override (primarily for testing)
 * @throws {Error} In production if VITE_API_BASE_URL is not configured
 * @returns {string} The normalized API base URL
 */
export function getApiBaseUrl(env: Environment = getCurrentEnvironment()): string {
  const configured = env.apiBaseUrl;

  // If explicitly configured, use it (works in both dev and prod)
  if (configured && configured.trim() !== '') {
    return normalizeBaseUrl(configured);
  }

  // Development: use intentional default that routes through Vite proxy
  if (isDevelopment(env)) {
    return '/api/v1';
  }

  // Production: fail fast with clear error
  throw new Error(
    'API base URL is not configured. ' +
      'Set VITE_API_BASE_URL environment variable for production builds. ' +
      'Example: VITE_API_BASE_URL=https://api.example.com/v1'
  );
}

/**
 * The configured API base URL.
 * This is computed once at module load time.
 */
export const API_BASE_URL = getApiBaseUrl();
