/**
 * @fileoverview Tests for API base URL configuration.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

describe('API base URL configuration', () => {
  const originalEnv = process.env;
  const originalImportMeta = (globalThis as any).importMeta;

  beforeEach(() => {
    vi.resetModules();
    process.env = { ...originalEnv };
    delete (globalThis as any).importMeta;
  });

  afterEach(() => {
    process.env = originalEnv;
    (globalThis as any).importMeta = originalImportMeta;
    vi.restore
  });

  it('should use configured VITE_API_BASE_URL when present', async () => {
    (globalThis as any).importMeta = {
      env: {
        VITE_API_BASE_URL: 'https://api.example.com/v2',
        DEV: false,
        PROD: true,
      },
    };

    const { getApiBaseUrl } = await import('./api');
    expect(getApiBaseUrl()).toBe('https://api.example.com/v2');
  });

  it('should normalize configured URL by removing trailing slashes', async () => {
    (globalThis as any).importMeta = {
      env: {
        VITE_API_BASE_URL: 'https://api.example.com/v2/',
        DEV: false,
        PROD: true,
      },
    };

    const { getApiBaseUrl } = await import('./api');
    expect(getApiBaseUrl()).toBe('https://api.example.com/v2');
  });

  configurations', async () => {
    (globalThis as any).importMeta = {
      env: {
        VITE_API_BASE_URL: 'https://api.example.com/v2/',
        DEV: false,
        PROD: true,
      },
    };

    const { getApiBaseUrl } = await import('./api');
    expect(getApiBaseUrl()).toBe('https://api.example.com/v2');
  });

  it('should throw in production when VITE_API_BASE_URL is missing', async () => {
    (globalThis as any).importMeta = {
      env: {
        DEV: false,
        PROD: true,
      },
    };

    const { getApiBaseUrl } = await import('./api');
    expect(() => getApiBaseUrl()).toThrow(
      /VITE_API_BASE_URL is required in production builds/
    );
  });

  it('should fall back to localhost in development when VITE_API_BASE_URL is missing', async () => {
    (globalThis as any).importMeta = {
      env: {
        DEV: true,
        PROD: false,
      },
    };

    const { getApiBaseUrl } = await import('./api');
    expect(getApiBaseUrl()).toBe('http://localhost:8080/api/v1');
  });
});