/**
 * @fileoverview Tests for API configuration with explicit environment validation.
 */

import { describe, it, expect } from 'vitest';
import { getApiBaseUrl, normalizeBaseUrl, isDevelopment, type Environment } from './apiConfig';

describe('apiConfig', () => {
  describe('development environment', () => {
    it('should use /api/v1 default when no VITE_API_BASE_URL is set', () => {
      const env: Environment = {
        mode: 'development',
        apiBaseUrl: undefined,
      };
      expect(getApiBaseUrl(env)).toBe('/api/v1');
    });

    it('should use configured URL when VITE_API_BASE_URL is set in development', () => {
      const env: Environment = {
        mode: 'development',
        apiBaseUrl: 'https://dev.example.com/api',
      };
      expect(getApiBaseUrl(env)).toBe('https://dev.example.com/api');
    });

    it('should normalize trailing slashes from configured URL', () => {
      const env: Environment = {
        mode: 'development',
        apiBaseUrl: 'https://dev.example.com/api/',
      };
      expect(getApiBaseUrl(env)).toBe('https://dev.example.com/api');
    });

    it('should normalize multiple trailing slashes', () => {
      const env: Environment = {
        mode: 'development',
        apiBaseUrl: 'https://dev.example.com/api///',
      };
      expect(getApiBaseUrl(env)).toBe('https://dev.example.com/api');
    });
  });

  describe('production environment', () => {
    it('should throw error when VITE_API_BASE_URL is not set', () => {
      const env: Environment = {
        mode: 'production',
        apiBaseUrl: undefined,
      };
      expect(() => getApiBaseUrl(env)).toThrow(
        /API base URL is not configured/
      );
    });

    it('should use configured URL when VITE_API_BASE_URL is set', () => {
      const env: Environment = {
        mode: 'production',
        apiBaseUrl: 'https://api.example.com/v1',
      };
      expect(getApiBaseUrl(env)).toBe('https://api.example.com/v1');
    });

    it('should normalize trailing slashes in production', () => {
      const env: Environment = {
        mode: 'production',
        apiBaseUrl: 'https://api.example.com/v1/',
      };
      expect(getApiBaseUrl(env)).toBe('https://api.example.com/v1');
    });
  });

  describe('staging environment', () => {
    it('should require explicit configuration in non-development modes', () => {
      const env: Environment = {
        mode: 'staging',
        apiBaseUrl: undefined,
      };
      expect(() => getApiBaseUrl(env)).toThrow(
        /API base URL is not configured/
      );
    });

    it('should use configured URL in staging', () => {
      const env: Environment = {
        mode: 'staging',
        apiBaseUrl: 'https://staging-api.example.com',
      };
      expect(getApiBaseUrl(env)).toBe('https://staging-api.example.com');
    });
  });

  describe('edge cases', () => {
    it('should handle empty string as missing configuration', () => {
      const env: Environment = {
        mode: 'production',
        apiBaseUrl: '',
      };
      expect(() => getApiBaseUrl(env)).toThrow(
        /API base URL is not configured/
      );
    });

    it('should handle whitespace-only string as missing configuration', () => {
      const env: Environment = {
        mode: 'production',
        apiBaseUrl: '   ',
      };
      expect(() => getApiBaseUrl(env)).toThrow(
        /API base URL is not configured/
      );
    });

    it('should handle URLs with ports', () => {
      const env: Environment = {
        mode: 'development',
        apiBaseUrl: 'http://localhost:8080/api/v1',
      };
      expect(getApiBaseUrl(env)).toBe('http://localhost:8080/api/v1');
    });

    it('should handle URLs with query parameters', () => {
      const env: Environment = {
        mode: 'development',
        apiBaseUrl: 'https://api.example.com/v1?version=2',
      };
      expect(getApiBaseUrl(env)).toBe('https://api.example.com/v1?version=2');
    });
  });

  describe('normalizeBaseUrl', () => {
    it('should remove single trailing slash', () => {
      expect(normalizeBaseUrl('https://api.example.com/')).toBe('https://api.example.com');
    });

    it('should remove multiple trailing slashes', () => {
      expect(normalizeBaseUrl('https://api.example.com///')).toBe('https://api.example.com');
    });

    it('should not modify URLs without trailing slashes', () => {
      expect(normalizeBaseUrl('https://api.example.com')).toBe('https://api.example.com');
    });

    it('should preserve path slashes', () => {
      expect(normalizeBaseUrl('https://api.example.com/v1/api')).toBe('https://api.example.com/v1/api');
    });
  });

  describe('isDevelopment', () => {
    it('should return true for development mode', () => {
      const env: Environment = { mode: 'development' };
      expect(isDevelopment(env)).toBe(true);
    });

    it('should return false for production mode', () => {
      const env: Environment = { mode: 'production' };
      expect(isDevelopment(env)).toBe(false);
    });

    it('should return false for staging mode', () => {
      const env: Environment = { mode: 'staging' };
      expect(isDevelopment(env)).toBe(false);
    });

    it('should return false for undefined mode', () => {
      const env: Environment = { mode: undefined };
      expect(isDevelopment(env)).toBe(false);
    });
  });
});
