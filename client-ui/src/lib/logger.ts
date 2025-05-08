const isDevelopment = process.env.NODE_ENV === 'development';

export type LogArgument = string | number | boolean | object;

export function info(...args: LogArgument[]) {
  if (isDevelopment) {
    console.log('[LOG]', ...args);
  }
}

export function warn(...args: LogArgument[]) {
  if (isDevelopment) {
    console.warn('[WARN]', ...args);
  }
}

export function error(...err: any[]) {
  console.error('[ERROR]', err);
}

export function debug(...args: LogArgument[]) {
  if (isDevelopment) {
    console.debug('[DEBUG]', ...args);
  }
}

const logger = {
  info: info,
  warn: warn,
  error: error,
  debug: debug,
};
export default logger;
