// scripts/check-i18n.ts
import en from '../src/i18n/locales/en.json';
import zh from '../src/i18n/locales/zh-CN.json';

function collectKeys(obj: unknown, prefix = ''): string[] {
  if (typeof obj !== 'object' || obj === null) return [];
  return Object.entries(obj).flatMap(([key, value]) => {
    const full = prefix ? `${prefix}.${key}` : key;
    if (typeof value === 'object' && value !== null) {
      return collectKeys(value, full);
    }
    return full;
  });
}

const enKeys = new Set(collectKeys(en));
const zhKeys = new Set(collectKeys(zh));

const missingInZh = [...enKeys].filter((k) => !zhKeys.has(k));
const missingInEn = [...zhKeys].filter((k) => !enKeys.has(k));

if (missingInZh.length || missingInEn.length) {
  console.error('i18n key mismatch:');
  missingInZh.forEach((k) => console.error(`  missing in zh-CN: ${k}`));
  missingInEn.forEach((k) => console.error(`  missing in en: ${k}`));
  process.exit(1);
}

console.log('i18n keys are consistent.');
