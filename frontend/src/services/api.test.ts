/**
 * @fileoverview Tests for API base URL configuration and validation.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

describe('API Base URL Configuration', () => {
  let originalEnv: typeof import.meta.env;

  beforeEach(() => {
    originalEnv = { ...import.meta.env };
  });

  afterEach(() => {
    vi.resetModules();
    Object.assign(import.meta.env, originalEnv);
  });

  it('should use configured VITE_API_BASE_URL when present', async () => {
    Object.defineProperty(import.meta, 'env', {
      value: {
        VITE_API_BASE_URL: 'https://api.example.com/v2',
        PROD: false,
      },
      writable: true,
      configurable: true,
    });

    const { getApiBaseUrl } = await import('./api');
    expect(getApiBaseUrl()).toBe('https://api.example.com/v2');
  });

  it('should normalize configured URLs with trailing slashes', async () => {
    Object.defineProperty(import.meta, 'env', {
      value: {
        VITE_API_BASE_URL: 'https://api.example.com/v2/',
        PROD: false,
      },
      writable: true,
      configurable: true,
    });

    const { getApiBaseUrl } = await import('./api');
    expect(getApiBaseUrl()).toBe('https://api.example.com/v2');
  });

  it('should throw configuration error in production when URL is missing', async () => {
    Object.defineProperty(import.meta, 'env', {
      value: {
        VITE_API_BASE_URL: undefined,
        PROD: true,
,ep      },
      writable: true,
      configurable: true,
    });

    const { getApiBaseUrl } = await import('./api');
    expect(() => getApiBaseUrl()).toThrow(
      '[API Config Error] VITE_API_BASE_URL is required in production builds'
    );
  });

  it('should return development default when no URL is configured in development', async () => {
    Object.defineProperty(import.meta, 'env', {
      value: {
        VITE_API_BASE_URL: undefined,
        PROD: false,
      },
      writable: true,
      configurable: true,
    });

    const { getApiBaseUrl } = await import('./api');
    expect(getApiBaseUrl()).toBe('http://localhost:8080/api/v1');
  });
});