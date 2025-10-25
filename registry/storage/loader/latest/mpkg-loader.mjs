import fs from 'fs';
import path from 'path';
import url from 'url';

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));
const resolveCache = new Map();

// Parse JSONC
function parseJsonC(str) {
  let inString = false, escaped = false, result = '';
  for (let i = 0; i < str.length; i++) {
    const char = str[i];
    if (char === '"' && !escaped) inString = !inString;
    if (!inString) {
      if (char === '/' && str[i + 1] === '/') {
        while (str[i] !== '\n' && i < str.length) i++;
        continue;
      }
      if (char === '/' && str[i + 1] === '*') {
        i += 2;
        while (!(str[i] === '*' && str[i + 1] === '/') && i < str.length) i++;
        i++; // skip '/'
        continue;
      }
    }
    escaped = char === '\\' && !escaped;
    result += char;
  }
  return JSON.parse(result);
}

// Read pkg.jsonc
const pkgPath = path.resolve(process.cwd(), 'pkg.jsonc');
const pkgJson = fs.existsSync(pkgPath)
  ? parseJsonC(fs.readFileSync(pkgPath, 'utf-8'))
  : { name: 'Anon', version: '0.0.0', dependencies: {} };

// Resolve main entry of a package
function resolvePackageEntry(pkgDir) {
  const pkgJsonPath = path.join(pkgDir, 'package.json');
  if (fs.existsSync(pkgJsonPath)) {
    const pkg = JSON.parse(fs.readFileSync(pkgJsonPath, 'utf-8'));
    if (pkg.main) {
      const mainPath = path.join(pkgDir, pkg.main);
      if (fs.existsSync(mainPath)) return mainPath;
    }
  }
  const indexJs = path.join(pkgDir, 'index.js');
  if (fs.existsSync(indexJs)) return indexJs;
  throw new Error(`Cannot resolve entry for package at ${pkgDir}`);
}

// Custom resolver
export async function resolve(specifier, context, defaultResolve) {
  const parentUrl = context.parentURL ? new URL(context.parentURL) : null;
  const cacheKey = (parentUrl?.href || '') + '|' + specifier;

  if (resolveCache.has(cacheKey)) {
    return { url: resolveCache.get(cacheKey), shortCircuit: true };
  }

  // Bare specifier in pkgJson.dependencies
  if (pkgJson.dependencies && pkgJson.dependencies[specifier]) {
    let packageDir = path.resolve(process.cwd(), 'packages', specifier);

    if (!fs.existsSync(packageDir)) {
      // fallback to node_modules
      packageDir = path.resolve(process.cwd(), 'packages', 'node_modules', specifier);
      if (!fs.existsSync(packageDir)) {
        throw new Error(`Package "${specifier}" not found`);
      }
    }

    const entryFile = resolvePackageEntry(packageDir);
    const finalUrl = url.pathToFileURL(entryFile).href;
    resolveCache.set(cacheKey, finalUrl);

    return { url: finalUrl, shortCircuit: true };
  }

  return defaultResolve(specifier, context);
}

// Load module
export async function load(urlStr, context, defaultLoad) {
  return defaultLoad(urlStr, context, defaultLoad);
}
